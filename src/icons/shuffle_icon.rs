// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

// Moosync
// Copyright (C) 2025 Moosync
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use leptos::{component, prelude::*, view, IntoView};

#[tracing::instrument(level = "trace", skip(filled))]
#[component]
pub fn ShuffleIcon<T>(#[prop()] filled: T) -> impl IntoView
where
    T: Get<Value = bool> + 'static + Copy + Send,
{
    view! {
        <svg
            class="button-grow"
            width="26"
            height="23"
            viewBox="0 0 26 23"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                fill-rule="evenodd"
                clip-rule="evenodd"
                d="M20.299 0.717278C20.371 0.645096 20.4566 0.587827 20.5507 0.548752C20.6449 0.509677 20.7459 0.489563 20.8478 0.489563C20.9498 0.489563 21.0507 0.509677 21.1449 0.548752C21.239 0.587827 21.3246 0.645096 21.3966 0.717278L25.2721 4.59278C25.3443 4.66478 25.4015 4.75031 25.4406 4.84448C25.4797 4.93865 25.4998 5.0396 25.4998 5.14155C25.4998 5.24351 25.4797 5.34446 25.4406 5.43862C25.4015 5.53279 25.3443 5.61832 25.2721 5.69032L21.3966 9.56583C21.251 9.71137 21.0536 9.79314 20.8478 9.79314C20.642 9.79314 20.4446 9.71137 20.299 9.56583C20.1535 9.42028 20.0717 9.22289 20.0717 9.01706C20.0717 8.81123 20.1535 8.61383 20.299 8.46828L23.6273 5.14155L20.299 1.81482C20.2269 1.74282 20.1696 1.65729 20.1305 1.56312C20.0914 1.46895 20.0713 1.368 20.0713 1.26605C20.0713 1.1641 20.0914 1.06315 20.1305 0.968979C20.1696 0.874811 20.2269 0.789278 20.299 0.717278ZM20.299 13.1189C20.371 13.0467 20.4566 12.9894 20.5507 12.9504C20.6449 12.9113 20.7459 12.8912 20.8478 12.8912C20.9498 12.8912 21.0507 12.9113 21.1449 12.9504C21.239 12.9894 21.3246 13.0467 21.3966 13.1189L25.2721 16.9944C25.3443 17.0664 25.4015 17.1519 25.4406 17.2461C25.4797 17.3403 25.4998 17.4412 25.4998 17.5432C25.4998 17.6451 25.4797 17.7461 25.4406 17.8402C25.4015 17.9344 25.3443 18.0199 25.2721 18.0919L21.3966 21.9674C21.251 22.113 21.0536 22.1947 20.8478 22.1947C20.642 22.1947 20.4446 22.113 20.299 21.9674C20.1535 21.8219 20.0717 21.6245 20.0717 21.4187C20.0717 21.2128 20.1535 21.0154 20.299 20.8699L23.6273 17.5432L20.299 14.2164C20.2269 14.1444 20.1696 14.0589 20.1305 13.9647C20.0914 13.8706 20.0713 13.7696 20.0713 13.6677C20.0713 13.5657 20.0914 13.4648 20.1305 13.3706C20.1696 13.2764 20.2269 13.1909 20.299 13.1189Z"
                fill=move || if !filled.get() { "var(--textSecondary)" } else { "var(--accent)" }
            ></path>
            <path
                fill-rule="evenodd"
                clip-rule="evenodd"
                d="M0.695312 5.14159C0.695312 4.93602 0.776975 4.73887 0.922334 4.59351C1.06769 4.44815 1.26484 4.36649 1.47041 4.36649H4.57082C9.30358 4.36649 11.6459 7.86684 13.6488 10.9176L13.7728 11.1083C14.7587 12.612 15.6703 13.9994 16.7678 15.0443C17.8808 16.1015 19.152 16.7681 20.8479 16.7681H23.9483C24.1539 16.7681 24.3511 16.8498 24.4964 16.9951C24.6418 17.1405 24.7234 17.3376 24.7234 17.5432C24.7234 17.7488 24.6418 17.9459 24.4964 18.0913C24.3511 18.2366 24.1539 18.3183 23.9483 18.3183H20.8479C18.6683 18.3183 17.0329 17.4347 15.6997 16.1666C14.4673 14.9962 13.4659 13.4662 12.514 12.0136L12.3513 11.7671C10.2849 8.61714 8.36416 5.91669 4.57082 5.91669H1.47041C1.26484 5.91669 1.06769 5.83502 0.922334 5.68967C0.776975 5.54431 0.695313 5.34716 0.695312 5.14159Z"
                fill=move || if !filled.get() { "var(--textSecondary)" } else { "var(--accent)" }
            ></path>
            <path
                fill-rule="evenodd"
                clip-rule="evenodd"
                d="M0.695312 17.5432C0.695313 17.7488 0.776975 17.9459 0.922334 18.0913C1.06769 18.2366 1.26484 18.3183 1.47041 18.3183H4.57082C9.30358 18.3183 11.6459 14.8179 13.6488 11.7671L13.7728 11.5765C14.7587 10.0728 15.6703 8.68535 16.7678 7.64051C17.8808 6.58327 19.152 5.91669 20.8479 5.91669H23.9483C24.1539 5.91669 24.3511 5.83502 24.4964 5.68967C24.6418 5.54431 24.7234 5.34716 24.7234 5.14159C24.7234 4.93602 24.6418 4.73887 24.4964 4.59351C24.3511 4.44815 24.1539 4.36649 23.9483 4.36649H20.8479C18.6683 4.36649 17.0329 5.2501 15.6997 6.51817C14.4673 7.68857 13.4659 9.21862 12.514 10.6712L12.3513 10.9176C10.2849 14.0676 8.36416 16.7681 4.57082 16.7681H1.47041C1.26484 16.7681 1.06769 16.8498 0.922334 16.9951C0.776975 17.1405 0.695313 17.3376 0.695312 17.5432Z"
                fill=move || if !filled.get() { "var(--textSecondary)" } else { "var(--accent)" }
            ></path>
        </svg>
    }
}
