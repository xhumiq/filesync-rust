use leptos::prelude::*;
use leptos_router::components::*;
use crate::i18n::{use_i18n, t, Locale};
use web_sys::window;

#[component]
pub fn MainTopNav() -> impl IntoView {
    let i18n = use_i18n();
    let (audio_dropdown_open, set_audio_dropdown_open) = signal(false);

    let toggle_language = move |_| {
        let current_locale = i18n.get_locale();
        let new_locale = match current_locale {
            Locale::en => Locale::zh,
            Locale::zh => Locale::fr,
            Locale::fr => Locale::en,
        };
        i18n.set_locale(new_locale);

        // Save to localStorage
        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let locale_str = match new_locale {
                    Locale::en => "en",
                    Locale::zh => "zh",
                    Locale::fr => "fr",
                };
                let _ = storage.set_item("locale", locale_str);
            }
        }
    };
    view! {
        {/* ==== TOP BAR ==== */}
        <div class="sticky top-0 z-50 flex items-center justify-center px-4 py-1 text-white bg-teal-700 top-bar">
            <div class="flex items-center">
                <h1 class="mr-6 text-xl font-bold"><A href="/">{t!(i18n, site_title)}</A></h1>
   
                <nav class="hidden space-x-6 md:flex">
                    <div class="dropdown dropdown-hover">
                        <div tabindex="0" role="button" class="text-white cursor-pointer hover:text-teal-200">
                            <A href="/ui/videos" attr:class="text-white hover:bg-teal-700">{t!(i18n, video)}</A>
                        </div>
                        <ul tabindex="0" class="dropdown-content menu bg-teal-600 text-white rounded-md z-[1] w-52 p-2 shadow">
                            <li><A href="/ui/videos/today" attr:class="text-white hover:bg-teal-700">{t!(i18n, today)}</A></li>
                            <li><A href="/ui/videos/3days" attr:class="text-white hover:bg-teal-700">{t!(i18n, past_3_days)}</A></li>
                            <li><A href="/ui/videos/date" attr:class="text-white hover:bg-teal-700">{t!(i18n, choose_date)}</A></li>
                            <li><A href="/files/Compressed/english" attr:class="text-white hover:bg-teal-700">{t!(i18n, compressed_english)}</A></li>
                            <li><A href="/files/Compressed/chinese" attr:class="text-white hover:bg-teal-700">{t!(i18n, compressed_chinese)}</A></li>
                        </ul>
                    </div>

                    <div class="dropdown" class:dropdown-open={move || audio_dropdown_open.get()} on:mouseenter=move |_| set_audio_dropdown_open.set(true) on:mouseleave=move |_| set_audio_dropdown_open.set(false)>
                        <div tabindex="0" role="button" class="text-white cursor-pointer hover:text-teal-200">
                            <A href="/ui/audio" attr:class="text-white hover:bg-teal-700">{t!(i18n, audio)}</A>
                        </div>
                        <ul tabindex="0" class="dropdown-content menu bg-teal-600 text-white rounded-md z-[1] w-52 p-2 shadow">
                            <li><A href="/ui/audio/this_week" attr:class="text-white hover:bg-teal-700" on:click=move |_| set_audio_dropdown_open.set(false)>{t!(i18n, this_week)}</A></li>
                            <li><A href="/ui/audio/date" attr:class="text-white hover:bg-teal-700" on:click=move |_| set_audio_dropdown_open.set(false)>{t!(i18n, choose_date)}</A></li>
                        </ul>
                    </div>
                    
                    <div class="dropdown dropdown-hover">
                        <div tabindex="0" role="button" class="text-white cursor-pointer hover:text-teal-200">
                            {t!(i18n, docs)}
                        </div>
                        <ul tabindex="0" class="dropdown-content menu bg-teal-600 text-white rounded-md z-[1] w-52 p-2 shadow">
                            <li><A href="/files/LiteraryCenter/SpiritualScripts/HPrayer" attr:class="text-white hover:bg-teal-700">{t!(i18n, grandpas_prayer)}</A></li>
                            <li><A href="/files/LiteraryCenter/SpiritualScripts/HMessage" attr:class="text-white hover:bg-teal-700">{t!(i18n, grandpas_message)}</A></li>
                            <li><A href="/files/LiteraryCenter/SpiritualScripts/OpenLetter" attr:class="text-white hover:bg-teal-700">{t!(i18n, open_letter)}</A></li>
                            <li><A href="/files/LiteraryCenter/SpiritualScripts/Other" attr:class="text-white hover:bg-teal-700">{t!(i18n, other)}</A></li>
                            <li><A href="/files/LiteraryCenter/AudioBooks/english" attr:class="text-white hover:bg-teal-700">{t!(i18n, audio_books)}</A></li>
                            <li><A href="/files/LiteraryCenter/DietRevolution/english" attr:class="text-white hover:bg-teal-700">{t!(i18n, diet_revolution)}</A></li>
                        </ul>
                    </div>
                   
                    <div class="dropdown dropdown-hover">
                        <div tabindex="0" role="button" class="text-white cursor-pointer hover:text-teal-200">
                            <A href="/ui/photos" attr:class="text-white hover:bg-teal-700">{t!(i18n, photos)}</A>
                        </div>
                        <ul tabindex="0" class="dropdown-content menu bg-teal-600 text-white rounded-md z-[1] w-52 p-2 shadow">
                            <li><A href="/ui/photos/this_week" attr:class="text-white hover:bg-teal-700">{t!(i18n, this_week)}</A></li>
                            <li><A href="/ui/photos/date" attr:class="text-white hover:bg-teal-700" on:click=move |_| set_audio_dropdown_open.set(false)>{t!(i18n, choose_date)}</A></li>
                        </ul>
                    </div>
                    
                    <div class="dropdown dropdown-hover">
                        <div tabindex="0" role="button" class="text-white cursor-pointer hover:text-teal-200">
                            {t!(i18n, hymns)}
                        </div>
                        <ul tabindex="0" class="dropdown-content menu bg-teal-600 text-white rounded-md z-[1] w-52 p-2 shadow">
                            <li><A href="/files/Hymns/mp3/Chinese" attr:class="text-white hover:bg-teal-700">{t!(i18n, mp3_chinese)}</A></li>
                            <li><A href="/files/Hymns/mp3/English" attr:class="text-white hover:bg-teal-700">{t!(i18n, mp3_english)}</A></li>
                            <li><A href="/files/Hymns/title/chinese" attr:class="text-white hover:bg-teal-700">{t!(i18n, titles_chinese)}</A></li>
                            <li><A href="/files/Hymns/title/chinese+english" attr:class="text-white hover:bg-teal-700">{t!(i18n, titles_chinese_and_english)}</A></li>
                            <li><A href="/files/Hymns/title/chinese+english+french" attr:class="text-white hover:bg-teal-700">{t!(i18n, titles_chinese_english_french)}</A></li>
                            <li><A href="/files/Hymns/lyrics/chinese" attr:class="text-white hover:bg-teal-700">{t!(i18n, sheet_music_chinese)}</A></li>
                            <li><A href="/files/Hymns/lyrics/english" attr:class="text-white hover:bg-teal-700">{t!(i18n, sheet_music_english)}</A></li>
                            <li><A href="/files/Hymns/video/dance" attr:class="text-white hover:bg-teal-700">{t!(i18n, dancing_tutorials)}</A></li>
                        </ul>
                    </div>
                    
                    <div class="dropdown dropdown-hover">
                        <div tabindex="0" role="button" class="text-white cursor-pointer hover:text-teal-200">
                            {t!(i18n, school)}
                        </div>
                        <ul tabindex="0" class="dropdown-content menu bg-teal-600 text-white rounded-md z-[1] w-52 p-2 shadow">
                            <li><A href="/files/Materials/Chinese" attr:class="text-white hover:bg-teal-700">{t!(i18n, elementary_chinese)}</A></li>
                            <li><A href="/files/Materials/English" attr:class="text-white hover:bg-teal-700">{t!(i18n, elementary_english)}</A></li>
                            <li><A href="/files/Materials/Math" attr:class="text-white hover:bg-teal-700">{t!(i18n, elementary_math)}</A></li>
                            <li><A href="/files/Materials/Nature" attr:class="text-white hover:bg-teal-700">{t!(i18n, elementary_science)}</A></li>
                            <li><A href="/files/Materials/Chinese" attr:class="text-white hover:bg-teal-700">{t!(i18n, junior_chinese)}</A></li>
                            <li><A href="/files/Materials/Chinese" attr:class="text-white hover:bg-teal-700">{t!(i18n, senior_chinese)}</A></li>
                            <li><A href="/files/Materials/Others" attr:class="text-white hover:bg-teal-700">{t!(i18n, others)}</A></li>
                        </ul>
                    </div>
                    
                    <div class="dropdown dropdown-hover">
                        <div tabindex="0" role="button" class="text-white cursor-pointer hover:text-teal-200">
                            {t!(i18n, graphics)}
                        </div>
                        <ul tabindex="0" class="dropdown-content menu bg-teal-600 text-white rounded-md z-[1] w-52 p-2 shadow">
                            <li><A href="/files/Graphics/backdrop" attr:class="text-white hover:bg-teal-700">{t!(i18n, banners)}</A></li>
                            <li><A href="/files/Graphics/bookmark" attr:class="text-white hover:bg-teal-700">{t!(i18n, bookmarks)}</A></li>
                            <li><A href="/files/Graphics/others" attr:class="text-white hover:bg-teal-700">{t!(i18n, others)}</A></li>
                            <li><A href="/files/Graphics/T-shirt" attr:class="text-white hover:bg-teal-700">{t!(i18n, tshirt)}</A></li>
                        </ul>
                    </div>
                </nav>
            </div>

            {/* Mobile Menu Button */}
            <div class="flex ml-auto space-x-2 md:hidden">
                <button class="text-white border-white btn btn-outline btn-sm">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
                    </svg>
                </button>
                <button class="text-white border-white btn btn-outline btn-sm" on:click=toggle_language>
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M5.636 5.636a9 9 0 1 0 12.728 0M12 3v9" />
                    </svg>
                </button>
            </div>
        </div>
    }
}
