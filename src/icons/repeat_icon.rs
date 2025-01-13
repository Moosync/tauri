use leptos::{component, prelude::*, view, IntoView};
use types::ui::player_details::RepeatModes;

#[tracing::instrument(level = "trace", skip(mode))]
#[component]
pub fn RepeatIcon<T>(#[prop()] mode: T) -> impl IntoView
where
    T: Get<Value = RepeatModes> + 'static + Copy + Send,
{
    view! {
        <div>
            {move || {
                match mode.get() {
                    RepeatModes::Once => {
                        view! {
                            <svg
                                width="27"
                                height="27"
                                viewBox="0 0 27 27"
                                fill="none"
                                class="button-grow"
                                xmlns="http://www.w3.org/2000/svg"
                            >
                                <path
                                    d="M26.0861 12.0223C25.7672 10.3767 24.9825 8.85737 23.8251 7.64494H23.8273L23.6698 7.48744C23.4576 7.29866 23.1803 7.19982 22.8966 7.21181C22.6128 7.2238 22.3449 7.34569 22.1494 7.5517C21.9539 7.7577 21.8462 8.03167 21.8491 8.31566C21.8519 8.59964 21.9652 8.87137 22.1648 9.07338C23.3808 10.3006 24.0618 11.9592 24.0592 13.6868L24.0482 14.0718C23.9495 15.7424 23.2162 17.3122 21.9984 18.4601C20.7806 19.608 19.1702 20.2473 17.4967 20.2471H10.2932H7.20009L6.87853 20.5687L6.75384 20.7196C6.60798 20.9301 6.54057 21.185 6.56336 21.44C6.58614 21.6951 6.69768 21.934 6.87853 22.1153L11.2579 26.4946L11.4088 26.6215C11.6195 26.7671 11.8746 26.8341 12.1296 26.8109C12.3847 26.7878 12.6235 26.6758 12.8045 26.4946L12.9313 26.3437C13.0769 26.133 13.144 25.8779 13.1208 25.6229C13.0976 25.3679 12.9857 25.129 12.8045 24.9481L10.2932 22.4346L17.4967 22.4368L17.932 22.4259C19.606 22.3423 21.2209 21.7797 22.5845 20.805C23.948 19.8302 25.0029 18.4843 25.6237 16.9273C26.2444 15.3703 26.4049 13.6678 26.0861 12.0223Z"
                                    fill="var(--accent)"
                                />
                                <path
                                    d="M14.8367 0.752128L14.9898 0.876816L19.367 5.25619L19.4938 5.40932C19.6199 5.59197 19.6874 5.80864 19.6874 6.03057C19.6874 6.25249 19.6199 6.46916 19.4938 6.65182L19.367 6.80494L19.0476 7.12432H15.9523H8.74665C7.07275 7.12422 5.46209 7.76378 4.24421 8.91212C3.02633 10.0605 2.29329 11.6308 2.19509 13.3018L2.18415 13.6868C2.18415 15.4806 2.90384 17.1037 4.06759 18.2893C4.25187 18.4997 4.34854 18.7728 4.33771 19.0523C4.32687 19.3318 4.20937 19.5965 4.00936 19.792C3.80935 19.9876 3.54202 20.099 3.26235 20.1035C2.98268 20.108 2.71191 20.0052 2.50572 19.8162C1.32454 18.6129 0.51619 17.094 0.177908 15.4421C-0.160375 13.7903 -0.0141796 12.0758 0.598906 10.5051C1.21199 8.93438 2.26585 7.57424 3.63371 6.58833C5.00157 5.60241 6.62516 5.03271 8.30915 4.94775L8.74665 4.93682L15.9523 4.93463L13.441 2.42557L13.3163 2.27244C13.1723 2.0619 13.1063 1.80761 13.1299 1.55359C13.1535 1.29956 13.2652 1.06178 13.4456 0.881389C13.626 0.700994 13.8638 0.589333 14.1178 0.565729C14.3718 0.542124 14.6261 0.60806 14.8367 0.752128Z"
                                    fill="var(--accent)"
                                />
                                <path
                                    d="M12.6178 12.0051H11.1178V10.5051L13.1271 10.5051L14.1178 10.5051V11.4982V16.5051H13.0272H12.6178V16.0367V12.0051Z"
                                    fill="var(--accent)"
                                />
                            </svg>
                        }
                            .into_any()
                    }
                    RepeatModes::None | RepeatModes::Loop => {
                        view! {
                            <svg
                                class="button-grow"
                                width="27"
                                height="27"
                                viewBox="0 0 27 27"
                                fill="none"
                                xmlns="http://www.w3.org/2000/svg"
                            >
                                <path
                                    class="fill"
                                    d="M24.2035 7.45811C25.3609 8.67054 26.1457 10.1899 26.4645 11.8354C26.7834 13.481 26.6229 15.1835 26.0021 16.7405C25.3813 18.2974 24.3264 19.6433 22.9629 20.6181C21.5993 21.5929 19.9845 22.1555 18.3104 22.239L17.8751 22.25L10.6716 22.2478L13.1829 24.7612C13.3641 24.9422 13.476 25.181 13.4992 25.4361C13.5224 25.6911 13.4553 25.9462 13.3098 26.1569L13.1829 26.3078C13.0019 26.489 12.7631 26.6009 12.5081 26.6241C12.253 26.6473 11.9979 26.5803 11.7873 26.4347L11.6363 26.3078L7.25695 21.9284C7.07609 21.7472 6.96456 21.5082 6.94178 21.2532C6.91899 20.9982 6.9864 20.7432 7.13226 20.5328L7.25695 20.3819L11.6363 16.0025C11.8292 15.8087 12.0877 15.6942 12.3609 15.6816C12.634 15.6689 12.902 15.7591 13.112 15.9342C13.322 16.1093 13.4587 16.3567 13.4953 16.6277C13.532 16.8987 13.4657 17.1735 13.3098 17.3981L13.1829 17.5512L10.6716 20.0603H17.8751C19.5486 20.0604 21.159 19.4212 22.3768 18.2733C23.5946 17.1254 24.3279 15.5556 24.4266 13.885L24.4376 13.5C24.4402 11.7723 23.7592 10.1138 22.5432 8.88655C22.3436 8.68454 22.2303 8.41281 22.2275 8.12883C22.2246 7.84484 22.3323 7.57087 22.5278 7.36487C22.7233 7.15886 22.9912 7.03697 23.275 7.02498C23.5587 7.01299 23.836 7.11183 24.0482 7.30061L24.2057 7.45811H24.2035ZM15.2151 0.565299L15.3682 0.689987L19.7454 5.06936L19.8723 5.22249C19.9983 5.40514 20.0658 5.62181 20.0658 5.84374C20.0658 6.06566 19.9983 6.28234 19.8723 6.46499L19.7454 6.61811L15.3682 10.9953L15.2151 11.1222C15.0324 11.2482 14.8157 11.3157 14.5938 11.3157C14.3719 11.3157 14.1552 11.2482 13.9726 11.1222L13.8194 10.9953L13.6948 10.8422C13.5687 10.6595 13.5012 10.4428 13.5012 10.2209C13.5012 9.999 13.5687 9.78232 13.6948 9.59967L13.8194 9.44655L16.3307 6.93749H9.12507C7.45117 6.93739 5.84051 7.57695 4.62263 8.7253C3.40474 9.87364 2.67171 11.444 2.57351 13.115L2.56257 13.5C2.56257 15.2937 3.28226 16.9169 4.44601 18.1025C4.63029 18.3129 4.72695 18.5859 4.71612 18.8654C4.70529 19.1449 4.58779 19.4097 4.38778 19.6052C4.18777 19.8007 3.92044 19.9122 3.64077 19.9167C3.36109 19.9212 3.09033 19.8184 2.88413 19.6294C1.70296 18.4261 0.894608 16.9071 0.556326 15.2553C0.218043 13.6034 0.364238 11.889 0.977324 10.3183C1.59041 8.74755 2.64427 7.38742 4.01213 6.4015C5.37999 5.41558 7.00357 4.84588 8.68757 4.76092L9.12507 4.74999L16.3307 4.7478L13.8194 2.23874L13.6948 2.08561C13.5507 1.87507 13.4848 1.62078 13.5084 1.36676C13.532 1.11274 13.6436 0.874956 13.824 0.69456C14.0044 0.514165 14.2422 0.402505 14.4962 0.3789C14.7502 0.355295 15.0045 0.421231 15.2151 0.565299Z"
                                    fill=move || {
                                        if mode.get() == RepeatModes::None {
                                            "var(--textSecondary)"
                                        } else {
                                            "var(--accent)"
                                        }
                                    }
                                ></path>
                            </svg>
                        }
                            .into_any()
                    }
                }
            }}
        </div>
    }
}
