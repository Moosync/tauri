use leptos::{component, view, IntoView};

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn SortIcon() -> impl IntoView {
    view! {
        <svg
            style="cursor: pointer;"
            width="23"
            height="22"
            viewBox="0 0 23 22"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M5.40504 17.1876C5.40504 17.3699 5.33261 17.5448 5.20368 17.6737C5.07475 17.8026 4.89988 17.8751 4.71754 17.8751C4.53521 17.8751 4.36034 17.8026 4.23141 17.6737C4.10248 17.5448 4.03004 17.3699 4.03004 17.1876V5.09719L2.45429 6.67432C2.39037 6.73824 2.31449 6.78895 2.23097 6.82354C2.14745 6.85813 2.05794 6.87594 1.96754 6.87594C1.87715 6.87594 1.78763 6.85813 1.70412 6.82354C1.6206 6.78895 1.54471 6.73824 1.48079 6.67432C1.41687 6.6104 1.36617 6.53451 1.33157 6.451C1.29698 6.36748 1.27917 6.27797 1.27917 6.18757C1.27917 6.09717 1.29698 6.00766 1.33157 5.92414C1.36617 5.84063 1.41687 5.76474 1.48079 5.70082L4.23079 2.95219L4.24042 2.94257C4.36967 2.81704 4.54319 2.7475 4.72336 2.74905C4.90353 2.75059 5.07584 2.82309 5.20292 2.95082L7.95292 5.70082C8.01684 5.76465 8.06756 5.84045 8.10219 5.92388C8.13682 6.00731 8.15468 6.09675 8.15474 6.18708C8.1548 6.27742 8.13707 6.36688 8.10256 6.45036C8.06805 6.53384 8.01744 6.60971 7.95361 6.67363C7.88978 6.73755 7.81398 6.78828 7.73055 6.8229C7.64711 6.85753 7.55768 6.87539 7.46734 6.87545C7.37701 6.87552 7.28755 6.85779 7.20406 6.82328C7.12058 6.78876 7.04471 6.73815 6.98079 6.67432L5.40504 5.09719V17.1876ZM10.2175 4.81257C10.2175 4.63023 10.29 4.45536 10.4189 4.32643C10.5478 4.1975 10.7227 4.12507 10.905 4.12507H20.53C20.7124 4.12507 20.8872 4.1975 21.0162 4.32643C21.1451 4.45536 21.2175 4.63023 21.2175 4.81257C21.2175 4.99491 21.1451 5.16977 21.0162 5.2987C20.8872 5.42764 20.7124 5.50007 20.53 5.50007H10.905C10.7227 5.50007 10.5478 5.42764 10.4189 5.2987C10.29 5.16977 10.2175 4.99491 10.2175 4.81257ZM10.905 8.25007C10.7227 8.25007 10.5478 8.3225 10.4189 8.45143C10.29 8.58036 10.2175 8.75523 10.2175 8.93757C10.2175 9.11991 10.29 9.29477 10.4189 9.42371C10.5478 9.55264 10.7227 9.62507 10.905 9.62507H17.78C17.9624 9.62507 18.1372 9.55264 18.2662 9.42371C18.3951 9.29477 18.4675 9.11991 18.4675 8.93757C18.4675 8.75523 18.3951 8.58036 18.2662 8.45143C18.1372 8.3225 17.9624 8.25007 17.78 8.25007H10.905ZM10.905 12.3751C10.7227 12.3751 10.5478 12.4475 10.4189 12.5764C10.29 12.7054 10.2175 12.8802 10.2175 13.0626C10.2175 13.2449 10.29 13.4198 10.4189 13.5487C10.5478 13.6776 10.7227 13.7501 10.905 13.7501H15.03C15.2124 13.7501 15.3872 13.6776 15.5162 13.5487C15.6451 13.4198 15.7175 13.2449 15.7175 13.0626C15.7175 12.8802 15.6451 12.7054 15.5162 12.5764C15.3872 12.4475 15.2124 12.3751 15.03 12.3751H10.905ZM10.905 16.5001C10.7227 16.5001 10.5478 16.5725 10.4189 16.7014C10.29 16.8304 10.2175 17.0052 10.2175 17.1876C10.2175 17.3699 10.29 17.5448 10.4189 17.6737C10.5478 17.8026 10.7227 17.8751 10.905 17.8751H12.28C12.4624 17.8751 12.6372 17.8026 12.7662 17.6737C12.8951 17.5448 12.9675 17.3699 12.9675 17.1876C12.9675 17.0052 12.8951 16.8304 12.7662 16.7014C12.6372 16.5725 12.4624 16.5001 12.28 16.5001H10.905Z"
                fill="var(--textPrimary)"
            ></path>
        </svg>
    }
}
