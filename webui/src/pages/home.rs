use crate::i18n::{use_i18n, t};
use leptos::prelude::*;
use crate::components::main_top_nav::MainTopNav;

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        {/* ==== TOP BAR ==== */}
        <MainTopNav />
        
        {/* Main Content */}
        <div class="container p-4 mx-auto">
            <div class="text-center">
                <h2 class="pb-2 mb-12 text-6xl font-bold text-gray-800 border-b-4 border-yellow-500 w-fit" style="font-family: 'Georgia';">
                    {t!(i18n, site_title)}
                </h2>
                
                <div class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
                    <div class="shadow-xl card bg-base-100" style="background: linear-gradient(to bottom, #d8d8d8 45%, #ffffff 45%);">
                        <div class="card-body">
                            <h3 class="pt-0 text-3xl card-title">{t!(i18n, ntc_video)}</h3>
                            <p>{t!(i18n, ntc_video_desc)}</p>
                        </div>
                    </div>
                    
                    <div class="shadow-xl card bg-base-100" style="background: linear-gradient(to bottom, #d8d8d8 45%, #ffffff 45%);">
                        <div class="card-body">
                            <h3 class="pt-0 text-3xl card-title">{t!(i18n, ntc_audio)}</h3>
                            <p>{t!(i18n, ntc_audio_desc)}</p>
                        </div>
                    </div>

                    <div class="shadow-xl card bg-base-100" style="background: linear-gradient(to bottom, #d8d8d8 45%, #ffffff 45%);">
                        <div class="card-body">
                            <h3 class="pt-0 text-3xl card-title">{t!(i18n, documents)}</h3>
                            <p>{t!(i18n, transcripts_and_audio_books_desc)}</p>
                        </div>
                    </div>
                    
                    <div class="shadow-xl card bg-base-100" style="background: linear-gradient(to bottom, #d8d8d8 45%, #ffffff 45%);">
                        <div class="card-body">
                            <h3 class="pt-0 text-3xl card-title">{t!(i18n, photos)}</h3>
                            <p>{t!(i18n, photos_desc)}</p>
                        </div>
                    </div>
                    
                    <div class="shadow-xl card bg-base-100" style="background: linear-gradient(to bottom, #d8d8d8 45%, #ffffff 45%);">
                        <div class="card-body">
                            <h3 class="pt-0 text-3xl card-title">{t!(i18n, hymns)}</h3>
                            <p>{t!(i18n, hymns_desc)}</p>
                        </div>
                    </div>

                    <div class="shadow-xl card bg-base-100" style="background: linear-gradient(to bottom, #d8d8d8 45%, #ffffff 45%);">
                        <div class="card-body">
                            <h3 class="pt-0 text-3xl card-title">{t!(i18n, graphic_images)}</h3>
                            <p>{t!(i18n, graphic_images_desc)}</p>
                        </div>
                    </div>
                    <div class="shadow-xl card bg-base-100" style="background: linear-gradient(to bottom, #d8d8d8 45%, #ffffff 45%);">
                        <div class="card-body">
                            <h3 class="pt-0 text-3xl card-title">{t!(i18n, educational_resources)}</h3>
                            <p>{t!(i18n, educational_resources_desc)}</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
