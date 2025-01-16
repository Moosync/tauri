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

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn SettingsIcon() -> impl IntoView {
    view! {
        <svg
            class="button-grow"
            width="19"
            height="18"
            viewBox="0 0 19 18"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M17.5189 7.4481C17.4882 7.27368 17.4191 7.10836 17.3166 6.96425C17.2142 6.82013 17.0809 6.70087 16.9266 6.61519L15.4066 5.76739C15.3091 5.57744 15.2029 5.39218 15.0882 5.21219L15.1174 3.46406C15.1209 3.28718 15.0849 3.11176 15.012 2.95068C14.9392 2.78959 14.8314 2.64693 14.6966 2.53315C13.9489 1.90041 13.0953 1.40605 12.1757 1.07308C12.01 1.01253 11.8329 0.989981 11.6575 1.00708C11.482 1.02418 11.3125 1.08049 11.1615 1.1719L9.6708 2.07067C9.45832 2.0605 9.24548 2.06033 9.03298 2.07015L7.54092 1.17063C7.39017 1.07917 7.22095 1.02278 7.04568 1.0056C6.87041 0.988421 6.69354 1.01089 6.52804 1.07137C5.60879 1.40561 4.75589 1.90115 4.00903 2.53493C3.87402 2.64882 3.76607 2.79161 3.69309 2.95284C3.62011 3.11407 3.58395 3.28965 3.58725 3.46674L3.6165 5.21323C3.50153 5.39306 3.39496 5.57818 3.29715 5.76799L1.77591 6.61645C1.62141 6.70229 1.488 6.82179 1.38546 6.96619C1.28292 7.11058 1.21385 7.27621 1.18333 7.45093C1.01229 8.41707 1.01312 9.40603 1.18578 10.3719C1.21645 10.5463 1.28555 10.7116 1.38803 10.8557C1.49051 10.9999 1.62377 11.1191 1.77805 11.2048L3.29803 12.0526C3.39553 12.2425 3.50181 12.4278 3.6165 12.6077L3.58725 14.3558C3.58381 14.5327 3.61981 14.7081 3.69263 14.8692C3.76545 15.0303 3.87323 15.173 4.00807 15.2868C4.75577 15.9195 5.60932 16.4139 6.529 16.7469C6.69468 16.8075 6.87175 16.83 7.04722 16.8129C7.2227 16.7958 7.39214 16.7394 7.54314 16.648L9.03387 15.7493C9.24628 15.7595 9.45972 15.7597 9.67169 15.7498L11.1637 16.6493C11.3145 16.7408 11.4837 16.7972 11.659 16.8144C11.8342 16.8316 12.0111 16.8091 12.1766 16.7487C13.0959 16.4144 13.9488 15.9189 14.6956 15.2851C14.8306 15.1712 14.9386 15.0284 15.0116 14.8672C15.0846 14.706 15.1207 14.5304 15.1174 14.3533L15.0882 12.6068C15.2031 12.427 15.3097 12.2419 15.4075 12.0521L16.9288 11.2035C17.0833 11.1177 17.2167 10.9982 17.3192 10.8538C17.4217 10.7094 17.4908 10.5438 17.5213 10.3691C17.6924 9.40294 17.6915 8.414 17.5189 7.44817V7.4481ZM9.35204 12.1842C8.70752 12.1842 8.07747 11.9921 7.54158 11.6324C7.00568 11.2726 6.588 10.7612 6.34135 10.1629C6.09471 9.56466 6.03017 8.90632 6.15591 8.27119C6.28165 7.63606 6.59202 7.05265 7.04776 6.59474C7.5035 6.13684 8.08415 5.825 8.71629 5.69867C9.34842 5.57233 10.0036 5.63717 10.5991 5.88499C11.1946 6.1328 11.7035 6.55247 12.0616 7.09091C12.4197 7.62935 12.6108 8.26238 12.6108 8.90995C12.6098 9.77802 12.2662 10.6103 11.6552 11.2241C11.0443 11.8379 10.216 12.1832 9.35204 12.1842V12.1842Z"
                fill="var(--textPrimary)"
            />
        </svg>
    }
}
