use leptos::prelude::*;
use leptos::reactive::wrappers::write::SignalSetter;
use leptos_router::components::*;
use web_sys::window;
use crate::i18n::{use_i18n, t, Locale};
use crate::langs::toggle_locale;
use crate::app_state::{ use_app_state, logout };
use crate::icons::*;

#[component]
pub fn MainTopNav() -> impl IntoView {
    let (audio_dropdown_open, set_audio_dropdown_open) = signal(false);
    let (menu_modal_open, set_menu_modal_open) = signal(false);

    let i18n = use_i18n();
    let current_locale = Memo::new(move |_| i18n.get_locale());
    let app_state = use_app_state();
    let app_state_stored = store_value(app_state.clone());
    let toggle_language = move |_| {
        toggle_locale(i18n, "");
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
                            <li><A href="/files/LiteraryCenter/Videos" attr:class="text-white hover:bg-teal-700">{t!(i18n, video_documentaries)}</A></li>
                        </ul>
                    </div>

                    <div class="dropdown" class:dropdown-open={move || audio_dropdown_open.get()} on:mouseenter=move |_| set_audio_dropdown_open.set(true) on:mouseleave=move |_| set_audio_dropdown_open.set(false)>
                        <div tabindex="0" role="button" class="text-white cursor-pointer hover:text-teal-200">
                            <A href="/ui/audio" attr:class="text-white hover:bg-teal-700">{t!(i18n, audio)}</A>
                        </div>
                        <ul tabindex="0" class="dropdown-content menu bg-teal-600 text-white rounded-md z-[1] w-52 p-2 shadow">
                            <li><A href="/ui/audio/this_week" attr:class="text-white hover:bg-teal-700" on:click=move |_| set_audio_dropdown_open.set(false)>{t!(i18n, this_week)}</A></li>
                            <li><A href="/ui/audio/date" attr:class="text-white hover:bg-teal-700" on:click=move |_| set_audio_dropdown_open.set(false)>{t!(i18n, choose_date)}</A></li>
                            <li><A href="/files/LiteraryCenter/AudioMessages" attr:class="text-white hover:bg-teal-700">{t!(i18n, recorded_messages)}</A></li>
                            <li><A href="/files/LiteraryCenter/AudioBooks/chinese" attr:class="text-white hover:bg-teal-700">{t!(i18n, audio_books_chinese)}</A></li>
                            <li><A href="/files/LiteraryCenter/AudioBooks/english" attr:class="text-white hover:bg-teal-700">{t!(i18n, audio_books_english)}</A></li>
                            <li><A href="/files/LiteraryCenter/AudioBooks/taiwanese" attr:class="text-white hover:bg-teal-700">{t!(i18n, audio_books_taiwanese)}</A></li>
                        </ul>
                    </div>
                    
                    <div class="dropdown dropdown-hover">
                        <div tabindex="0" role="button" class="text-white cursor-pointer hover:text-teal-200">
                            {t!(i18n, docs)}
                        </div>
                        <ul tabindex="0" class="dropdown-content menu bg-teal-600 text-white rounded-md z-[1] w-52 p-2 shadow">
                            <li><A href="/files/LiteraryCenter/SpiritualScripts/AudioTranscript" attr:class="text-white hover:bg-teal-700">{t!(i18n, audio_transcripts)}</A></li>
                            <li><A href="/files/LiteraryCenter/SpiritualBooks" attr:class="text-white hover:bg-teal-700">{t!(i18n, spiritual_books_chinese)}</A></li>
                            <li><A href="/files/LiteraryCenter/SpiritualBooks/O-English" attr:class="text-white hover:bg-teal-700">{t!(i18n, spiritual_books_english)}</A></li>
                            <li><A href="/files/LiteraryCenter/SpiritualScripts/HPrayer" attr:class="text-white hover:bg-teal-700">{t!(i18n, grandpas_prayer)}</A></li>
                            <li><A href="/files/LiteraryCenter/SpiritualScripts/HMessage" attr:class="text-white hover:bg-teal-700">{t!(i18n, grandpas_message)}</A></li>
                            <li><A href="/files/LiteraryCenter/SpiritualScripts/OpenLetter" attr:class="text-white hover:bg-teal-700">{t!(i18n, open_letter)}</A></li>
                            <li><A href="/files/LiteraryCenter/TruthEdification" attr:class="text-white hover:bg-teal-700">{t!(i18n, truth_edification)}</A></li>
                            <li><A href="/files/LiteraryCenter/SpiritualScripts/Other" attr:class="text-white hover:bg-teal-700">{t!(i18n, other)}</A></li>
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
                            <li><A href="/files/Graphics/others" attr:class="text-white hover:bg-teal-700">{t!(i18n, other_graphics)}</A></li>
                            <li><A href="/files/Graphics/T-shirt" attr:class="text-white hover:bg-teal-700">{t!(i18n, tshirt)}</A></li>
                        </ul>
                    </div>
                </nav>
            </div>

            {/* Mobile Menu Button */}
            <div class="absolute flex space-x-2 right-4">
                <button class="text-white border-white btn btn-outline btn-sm" on:click=toggle_language>
                    {move || {
                        match current_locale.get() {
                            Locale::en => view! {
                                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="size-6">
                                    <text x="50%" y="50%" text-anchor="middle" dy=".3em" font-size="12" font-weight="bold" fill="currentColor">En</text>
                                </svg>
                            }.into_any(),
                            Locale::fr => view! {
                                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="size-6">
                                    <text x="50%" y="50%" text-anchor="middle" dy=".3em" font-size="12" font-weight="bold" fill="currentColor">Fr</text>
                                </svg>
                            }.into_any(),
                            Locale::zh => view! {
                                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="size-6">
                                    <text x="50%" y="50%" text-anchor="middle" dy=".3em" font-size="14" font-weight="bold" fill="currentColor">文</text>
                                </svg>
                            }.into_any(),
                        }
                    }}
                </button>
                <button class="text-white border-white btn btn-outline btn-sm" on:click=move |_| {
                    set_menu_modal_open.update(|open| *open = !*open);
                }>
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
                    </svg>
                </button>
            </div>
        </div>

        {/* Mobile Menu Modal */}
        <Show when=move || menu_modal_open.get()>
            <div class="fixed inset-0 z-[60] flex items-center justify-center p-4">
                {/* Backdrop */}
                <div
                    class="absolute inset-0 bg-black bg-opacity-50"
                    on:click=move |_| {
                        set_menu_modal_open.set(false);
                    }
                />
                
                {/* Modal Content */}
                <div class="relative w-full max-w-sm max-h-[90vh] overflow-y-auto bg-white shadow-xl rounded-lg">
                    <div class="flex items-center justify-between border-b border-gray-300" style="position:relative">
                        <h2 class="p-2 text-xl font-bold">{t!(i18n, site_title)}</h2>
                        <button class="m-2 text-2xl" style="display:block;position:absolute;right:0;padding-right: 1rem" on:click=move |_| set_menu_modal_open.set(false)>"×"</button>
                    </div>

                    <div id="menu_modal_items" class="p-4 text-left">
                        <A href="/ui/videos" attr::class="block p-2 text-black hover:bg-gray-300 hover:text-white" on:click=move |_| set_menu_modal_open.set(false)>
                            <span class="icon">{home_icon()}</span>
                            {t!(i18n, video)}
                        </A>
                        <A href="/ui/videos/today" attr::class="block p-2 text-black hover:bg-gray-300 hover:text-white" on:click=move |_| set_menu_modal_open.set(false)>
                            <span class="icon">{rss_icon()}</span>
                            {t!(i18n, compressed_chinese)}
                        </A>
                        <A href="/ui/videos/3days" attr::class="block p-2 text-black hover:bg-gray-300 hover:text-white" on:click=move |_| set_menu_modal_open.set(false)>
                            <span class="icon">{rss_icon()}</span>
                            {t!(i18n, compressed_chieng)}
                        </A>
                        <A href="/ui/videos/date" attr::class="block p-2 text-black hover:bg-gray-300 hover:text-white" on:click=move |_| set_menu_modal_open.set(false)>
                            <span class="icon">{rss_icon()}</span>
                            {t!(i18n, compressed_english)}
                        </A>
                        <A href="/files/Compressed/english" attr::class="block p-2 text-black hover:bg-gray-300 hover:text-white" on:click=move |_| set_menu_modal_open.set(false)>
                            {t!(i18n, compressed_english)}
                        </A>
                        <A href="/files/Compressed/chinese" attr::class="block p-2 text-black hover:bg-gray-300 hover:text-white" on:click=move |_| set_menu_modal_open.set(false)>
                            {t!(i18n, compressed_chinese)}
                        </A>
                        <A href="/files/LiteraryCenter/Videos" attr::class="block p-2 text-black hover:bg-gray-300 hover:text-white" on:click=move |_| set_menu_modal_open.set(false)>
                            {t!(i18n, video_documentaries)}
                        </A>
                        <A href="/ui/audio" attr::class="block p-2 text-black hover:bg-gray-300 hover:text-white" on:click=move |_| set_menu_modal_open.set(false)>
                            {t!(i18n, audio)}
                        </A>
                        <A href="/ui/photos" attr::class="block p-2 text-black hover:bg-gray-300 hover:text-white" on:click=move |_| set_menu_modal_open.set(false)>
                            {t!(i18n, photos)}
                        </A>
                        <A href="/account/login" attr::class="block p-2 text-black hover:bg-gray-300 hover:text-white" on:click=move |ev| {
                            set_menu_modal_open.set(false);
                            logout(&app_state_stored.get_value());
                        }>
                            {t!(i18n, logout)}
                        </A>
                    </div>
                </div>
            </div>
        </Show>
    }
}
