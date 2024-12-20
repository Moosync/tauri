#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SongSortByColumns {
    Album,
    Artist,
    Date,
    Genre,
    PlayCount,
    Title,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PlaylistSortByColumns {
    Title,
    Provider,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct SongSortBy {
    pub asc: bool,
    pub sort_by: SongSortByColumns,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct PlaylistSortBy {
    pub asc: bool,
    pub sort_by: PlaylistSortByColumns,
}

pub struct UiStore {
    song_sort_by: SongSortBy,
    playlist_sort_by: PlaylistSortBy,
    show_queue: bool,
}

impl UiStore {
    #[tracing::instrument(level = "trace", skip())]
    pub fn new() -> Self {
        Self {
            song_sort_by: SongSortBy {
                asc: true,
                sort_by: SongSortByColumns::Album,
            },
            playlist_sort_by: PlaylistSortBy {
                asc: true,
                sort_by: PlaylistSortByColumns::Provider,
            },
            show_queue: false,
        }
    }

    #[tracing::instrument(level = "trace", skip(self, sort_by))]
    pub fn set_song_sort_by(&mut self, sort_by: SongSortBy) {
        self.song_sort_by = sort_by;
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_song_sort_by(&self) -> SongSortBy {
        self.song_sort_by
    }

    #[tracing::instrument(level = "trace", skip(self, sort_by))]
    pub fn set_playlist_sort_by(&mut self, sort_by: PlaylistSortBy) {
        self.playlist_sort_by = sort_by;
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_playlist_sort_by(&self) -> PlaylistSortBy {
        self.playlist_sort_by
    }
    pub fn show_queue(&mut self, show: bool) {
        self.show_queue = show;
    }

    pub fn get_show_queue(&self) -> bool {
        self.show_queue
    }

    pub fn toggle_show_queue(&mut self) {
        self.show_queue = !self.show_queue;
    }
}
