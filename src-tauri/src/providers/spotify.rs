use std::collections::HashSet;

use async_trait::async_trait;

use chrono::{DateTime, TimeDelta};
use futures::{channel::mpsc::UnboundedSender, SinkExt};
use oauth2::{CsrfToken, PkceCodeVerifier, TokenResponse};
use preferences::preferences::PreferenceConfig;
use regex::Regex;
use rspotify::{
    clients::{BaseClient, OAuthClient},
    model::{
        FullArtist, FullTrack, Id, PlaylistId, PlaylistTracksRef, SearchType, SimplifiedAlbum,
        SimplifiedArtist, SimplifiedPlaylist, TrackId,
    },
    AuthCodePkceSpotify, Token,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, State};
use types::{
    entities::{EntityInfo, QueryableAlbum, QueryableArtist, QueryablePlaylist, SearchResult},
    errors::errors::Result,
    oauth::OAuth2Client,
    providers::generic::{Pagination, ProviderStatus},
    songs::{QueryableSong, Song, SongType},
};
use types::{errors::errors::MoosyncError, providers::generic::GenericProvider};
use url::Url;

use crate::oauth::handler::OAuthHandler;

use super::common::{
    authorize, get_oauth_client, login, refresh_login, LoginArgs, OAuthClientArgs, TokenHolder,
};

macro_rules! search_and_parse_all {
    ($client:expr, $term:expr, [$(($type:expr, $variant:path, $parse_fn:expr, $result_vec:expr)),*]) => {{
        $(
            if let Ok(search_results) = $client.search($term, $type, None, None, Some(50), Some(0)).await {
                if let $variant(items) = search_results {
                    for item in items.items {
                        $parse_fn(item, &mut $result_vec);
                    }
                }
            }
        )*
    }};
}

#[derive(Debug, Clone, Default)]
struct SpotifyConfig {
    client_secret: Option<String>,
    client_id: Option<String>,
    redirect_uri: &'static str,
    scopes: Vec<&'static str>,
    tokens: Option<TokenHolder>,
}

#[derive(Debug)]
pub struct SpotifyProvider {
    app: AppHandle,
    config: SpotifyConfig,
    verifier: Option<(OAuth2Client, PkceCodeVerifier, CsrfToken)>,
    api_client: Option<AuthCodePkceSpotify>,
    status_tx: UnboundedSender<ProviderStatus>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ArtistExtraInfo {
    artist_id: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SpotifyExtraInfo {
    spotify: ArtistExtraInfo,
}

impl SpotifyProvider {
    pub fn new(app: AppHandle, status_tx: UnboundedSender<ProviderStatus>) -> Self {
        Self {
            app,
            config: SpotifyConfig::default(),
            verifier: None,
            api_client: None,
            status_tx,
        }
    }
}

impl SpotifyProvider {
    fn get_oauth_client(&self) -> OAuth2Client {
        get_oauth_client(OAuthClientArgs {
            auth_url: "https://accounts.spotify.com/authorize".to_string(),
            token_url: "https://accounts.spotify.com/api/token".to_string(),
            redirect_url: self.config.redirect_uri.to_string(),
            client_id: self.config.client_id.clone().unwrap(),
            client_secret: self.config.client_secret.clone().unwrap(),
        })
    }

    async fn create_api_client(&mut self) {
        if let Some(token) = &self.config.tokens {
            self.api_client = Some(AuthCodePkceSpotify::from_token(Token {
                access_token: token.access_token.clone(),
                expires_in: TimeDelta::seconds(token.expires_in.try_into().unwrap()),
                expires_at: Some(DateTime::from_timestamp_millis(token.expires_at).unwrap()),
                refresh_token: Some(token.refresh_token.clone()),
                scopes: HashSet::from_iter(self.config.scopes.iter().map(|v| v.to_string())),
            }));

            let res = self.fetch_user_details().await;
            if let Ok(res) = res {
                let _ = self.status_tx.send(res).await;
            } else {
                let _ = self
                    .status_tx
                    .send(ProviderStatus {
                        key: self.key(),
                        name: "Spotify".into(),
                        user_name: None,
                        logged_in: true,
                        bg_color: "#07C330".into(),
                        account_id: "spotify".into(),
                    })
                    .await;
            }
        }
    }

    async fn refresh_login(&mut self) -> Result<()> {
        self.config.tokens = Some(
            refresh_login(
                "MoosyncSpotifyRefreshToken",
                self.get_oauth_client(),
                &self.app,
            )
            .await?,
        );
        self.create_api_client().await;

        Ok(())
    }

    fn parse_playlist(&self, playlist: SimplifiedPlaylist) -> QueryablePlaylist {
        QueryablePlaylist {
            playlist_id: Some(format!("spotify-playlist:{}", playlist.id.id())),
            playlist_name: playlist.name,
            playlist_coverpath: playlist.images.first().map(|i| i.url.clone()),
            playlist_song_count: playlist.tracks.total as f64,
            extension: Some(self.key()),
            ..Default::default()
        }
    }

    fn parse_artists(&self, artist: SimplifiedArtist) -> QueryableArtist {
        QueryableArtist {
            artist_id: Some(format!("spotify-artist:{}", artist.id.clone().unwrap())),
            artist_name: Some(artist.name),
            artist_extra_info: Some(EntityInfo(
                serde_json::to_value(SpotifyExtraInfo {
                    spotify: ArtistExtraInfo {
                        artist_id: artist.id.clone().unwrap().to_string(),
                    },
                })
                .unwrap(),
            )),
            ..Default::default()
        }
    }

    fn parse_album(&self, album: SimplifiedAlbum) -> QueryableAlbum {
        QueryableAlbum {
            album_id: Some(format!("spotify-album:{}", album.id.clone().unwrap())),
            album_name: Some(album.name),
            album_artist: album.artists.first().map(|a| a.name.clone()),
            album_coverpath_high: album.images.first().map(|i| i.url.clone()),
            album_coverpath_low: album.images.last().map(|i| i.url.clone()),
            ..Default::default()
        }
    }

    fn parse_playlist_item(&self, item: FullTrack) -> Song {
        let id = item.id.unwrap().to_string();
        Song {
            song: QueryableSong {
                _id: Some(format!("spotify:{}", id)),
                title: Some(item.name),
                duration: Some(item.duration.num_seconds() as f64),
                type_: SongType::SPOTIFY,
                url: Some(id.clone()),
                song_cover_path_high: item.album.images.first().map(|i| i.url.clone()),
                playback_url: Some(id),
                track_no: Some(item.disc_number as f64),
                provider_extension: Some(self.key()),
                ..Default::default()
            },
            album: Some(self.parse_album(item.album)),
            artists: Some(
                item.artists
                    .into_iter()
                    .map(|a| self.parse_artists(a))
                    .collect(),
            ),
            ..Default::default()
        }
    }

    async fn fetch_user_details(&self) -> Result<ProviderStatus> {
        println!("Fetchinf user details {:?}", self.api_client);
        if let Some(api_client) = &self.api_client {
            let token = api_client.token.lock().await.unwrap();
            drop(token);

            let user = api_client.current_user().await?;
            return Ok(ProviderStatus {
                key: self.key(),
                name: "Spotify".into(),
                user_name: user.display_name,
                logged_in: true,
                bg_color: "#07C330".into(),
                account_id: "spotify".into(),
            });
        }

        Err("API client not initialized".into())
    }
}

#[async_trait]
impl GenericProvider for SpotifyProvider {
    async fn initialize(&mut self) -> Result<()> {
        let _ = self
            .status_tx
            .send(ProviderStatus {
                key: self.key(),
                name: "Spotify".into(),
                user_name: None,
                logged_in: false,
                bg_color: "#07C330".into(),
                account_id: "spotify".into(),
            })
            .await;

        let preferences: State<PreferenceConfig> = self.app.state();
        let spotify_config: Value = preferences.inner().load_selective("spotify".into())?;
        let client_id = spotify_config.get("client_id");
        let client_secret = spotify_config.get("client_secret");

        self.config.client_id = client_id.map(|v| v.as_str().unwrap().to_string());
        self.config.client_secret = client_secret.map(|v| v.as_str().unwrap().to_string());
        self.config.redirect_uri = "https://moosync.app/spotify";
        self.config.scopes = vec![
            "playlist-read-private",
            "user-top-read",
            "user-library-read",
            "user-read-private",
        ];

        let res = self.refresh_login().await;
        if let Err(err) = res {
            println!("spotify refresh login err: {:?}", err);
        }

        Ok(())
    }

    fn key(&self) -> String {
        "spotify".into()
    }

    fn match_id(&self, id: String) -> bool {
        id.starts_with("spotify-playlist:")
            || id.starts_with("spotify-artist:")
            || id.starts_with("spotify-album:")
            || id.starts_with("spotify:")
    }

    async fn login(&mut self, _: String) -> Result<()> {
        self.verifier = login(
            LoginArgs {
                client_id: self.config.client_id.clone(),
                client_secret: self.config.client_secret.clone(),
                scopes: self.config.scopes.clone(),
                extra_params: None,
            },
            self.get_oauth_client(),
            &self.app,
        )?;

        let oauth_handler: State<OAuthHandler> = self.app.state();
        oauth_handler.register_oauth_path("spotifyoauthcallback".into(), self.key());

        Ok(())
    }

    async fn signout(&mut self, _: String) -> Result<()> {
        self.config.tokens = None;
        self.api_client = None;
        self.verifier = None;

        let preferences: State<PreferenceConfig> = self.app.state();
        preferences.set_secure("MoosyncSpotifyRefreshToken".into(), None::<String>)?;

        let _ = self
            .status_tx
            .send(ProviderStatus {
                key: self.key(),
                name: "Spotify".into(),
                user_name: None,
                logged_in: false,
                bg_color: "#07C330".into(),
                account_id: "spotify".into(),
            })
            .await;
        Ok(())
    }

    async fn authorize(&mut self, code: String) -> Result<()> {
        println!("Authorizing with code {}", code);
        self.config.tokens = Some(
            authorize(
                "MoosyncSpotifyRefreshToken",
                code,
                &mut self.verifier,
                &self.app,
            )
            .await?,
        );

        self.create_api_client().await;
        Ok(())
    }

    async fn fetch_user_playlists(
        &self,
        pagination: Pagination,
    ) -> Result<(Vec<QueryablePlaylist>, Pagination)> {
        let mut ret = vec![];
        println!("Fetching spotify playlists {:?}", self.api_client);
        if let Some(api_client) = &self.api_client {
            let playlists = api_client
                .current_user_playlists_manual(Some(pagination.limit), Some(pagination.offset))
                .await;
            if let Ok(playlists) = playlists {
                for playlist in playlists.items {
                    ret.push(self.parse_playlist(playlist))
                }
            }
            println!("Got user playlists {:?}", ret);
            return Ok((ret, pagination.next_page()));
        }

        Err("API client not initialized".into())
    }

    async fn get_playlist_content(
        &self,
        playlist_id: String,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)> {
        let mut ret = vec![];
        if let Some(api_client) = &self.api_client {
            let playlist_id = playlist_id
                .strip_prefix("spotify-playlist:")
                .unwrap_or(&playlist_id);
            let items = api_client
                .playlist_items_manual(
                    PlaylistId::from_id_or_uri(playlist_id).unwrap(),
                    None,
                    None,
                    Some(pagination.limit),
                    Some(pagination.offset),
                )
                .await;
            if let Ok(items) = items {
                for i in items.items {
                    if i.is_local {
                        continue;
                    }

                    match i.track.unwrap() {
                        rspotify::model::PlayableItem::Track(t) => {
                            ret.push(self.parse_playlist_item(t));
                        }
                        rspotify::model::PlayableItem::Episode(_) => {
                            continue;
                        }
                    }
                }
            }
            return Ok((ret, pagination.next_page()));
        }
        Err("API client not initialized".into())
    }

    async fn get_playback_url(&self, _: Song, _: String) -> Result<String> {
        Err(MoosyncError::SwitchProviders("youtube".into()))
    }

    async fn search(&self, term: String) -> Result<SearchResult> {
        let mut ret = SearchResult {
            songs: vec![],
            albums: vec![],
            artists: vec![],
            playlists: vec![],
            ..Default::default()
        };

        if let Some(api_client) = &self.api_client {
            search_and_parse_all!(
                api_client,
                &term,
                [
                    (
                        SearchType::Track,
                        rspotify::model::SearchResult::Tracks,
                        |item, vec: &mut Vec<Song>| vec.push(self.parse_playlist_item(item)),
                        ret.songs
                    ),
                    (
                        SearchType::Playlist,
                        rspotify::model::SearchResult::Playlists,
                        |item, vec: &mut Vec<QueryablePlaylist>| vec
                            .push(self.parse_playlist(item)),
                        ret.playlists
                    ),
                    (
                        SearchType::Artist,
                        rspotify::model::SearchResult::Artists,
                        |item: FullArtist, vec: &mut Vec<QueryableArtist>| vec.push(
                            self.parse_artists(SimplifiedArtist {
                                external_urls: item.external_urls,
                                href: Some(item.href),
                                id: Some(item.id),
                                name: item.name,
                            })
                        ),
                        ret.artists
                    ),
                    (
                        SearchType::Album,
                        rspotify::model::SearchResult::Albums,
                        |item, vec: &mut Vec<QueryableAlbum>| vec.push(self.parse_album(item)),
                        ret.albums
                    )
                ]
            );
            return Ok(ret);
        }
        Err("API client not initialized".into())
    }

    async fn match_url(&self, url: String) -> Result<bool> {
        let re = Regex::new(
            r"^(https:\/\/open.spotify.com\/(track|embed)\/|spotify:track:)([a-zA-Z0-9]+)(.*)$",
        )
        .unwrap();
        if re.is_match(url.as_str()) {
            return Ok(true);
        }

        let re = Regex::new(
            r"^(https:\/\/open.spotify.com\/playlist\/|spotify:playlist:)([a-zA-Z0-9]+)(.*)$",
        )
        .unwrap();
        if re.is_match(url.as_str()) {
            return Ok(true);
        }
        Ok(false)
    }

    async fn playlist_from_url(&self, url: String) -> Result<QueryablePlaylist> {
        let playlist_id = Url::parse(url.as_str());
        let playlist_id = if let Ok(playlist_id) = playlist_id {
            playlist_id.path().to_string()
        } else {
            url
        };

        if let Some(api_client) = &self.api_client {
            let playlists = api_client
                .playlist(
                    PlaylistId::from_id_or_uri(playlist_id.as_str())
                        .map_err(|_| MoosyncError::String("Invalid playlist url".into()))?,
                    None,
                    None,
                )
                .await?;

            let res = self.parse_playlist(SimplifiedPlaylist {
                collaborative: playlists.collaborative,
                external_urls: playlists.external_urls,
                href: playlists.href,
                id: playlists.id,
                images: playlists.images,
                name: playlists.name,
                owner: playlists.owner,
                public: playlists.public,
                snapshot_id: playlists.snapshot_id,
                tracks: PlaylistTracksRef::default(),
            });

            return Ok(res);
        }

        Err("API Client not initialized".into())
    }

    async fn song_from_url(&self, url: String) -> Result<Song> {
        let track_id = Url::parse(url.as_str());
        let track_id = if let Ok(track_id) = track_id {
            track_id.path().to_string()
        } else {
            url
        };

        if let Some(api_client) = &self.api_client {
            let res = api_client
                .track(TrackId::from_id_or_uri(track_id.as_str())?, None)
                .await?;

            return Ok(self.parse_playlist_item(res));
        }

        Err("API Client not initialized".into())
    }
}
