//! Components and server functions to show transcripitions that are todo

use leptos::prelude::*;

use crate::app::TopLevelPosition;

#[component]
pub fn TranscribeTodoList() -> impl IntoView {
    let set_top_level_pos =
        use_context::<WriteSignal<TopLevelPosition>>().expect("App provides TopLevelPosition");
    *set_top_level_pos.write() = TopLevelPosition::Transcribe;

    view! {
    <div class="flex h-full flex-col">
      <div class="flex flex-row justify-center">
        <h1 class="text-6xl font-semibold p-10">Start a new Transcription for ...</h1>
      </div>
      <div class="flex flex-row justify-center mb-2">
        <div class="flex w-2/5 flex-row justify-start rounded-4xl border-2 border-slate-600 bg-slate-800 p-4 text-xl shadow-sky-600 shadow-md">
          <label for="page-search">
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6 text-slate-300">
              <path stroke-linecap="round" stroke-linejoin="round" d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z" /></svg
          ></label>
          <input id="page-search" class="w-0 grow border-0 font-mono text-slate-400" type="search" />
        </div>
      </div>
      <div class="mt-8 flex min-h-24 grow flex-row justify-center overflow-y-auto mb-10 no-scrollbar">
        <div id="page-listing" class="text-md table w-4/5">
          <div class="table-row-group">
            <a href="/transcribe/:msname/:pagename" class="table-row border-b border-slate-600 py-3 text-xl shadow-sky-600 last:border-b-0 odd:bg-slate-800 even:bg-slate-600 hover:bg-sky-900 hover:shadow-2xl">
              <div class="table-cell border-r border-inherit p-2">ML115</div>
              <div class="table-cell border-r border-inherit p-2">Page 014</div>
              <div class="table-cell border-r border-inherit p-2">Ps 32:4 - Ps 34:7</div>
              <div class="table-cell border-r border-inherit p-2 text-green-500">
                <div class="inline-flex translate-y-2">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8 translate-x-1">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
                  </svg>
                  <span class="pr-2 pl-4 font-extrabold">2</span>already done
                </div>
              </div>
              <div class="table-cell border-r border-inherit p-2 font-bold">already started</div>
              <div class="table-cell border-r border-inherit p-2"></div>
            </a>
            <div class="table-row border-b border-slate-600 py-3 text-xl shadow-sky-600 last:border-b-0 odd:bg-slate-800 even:bg-slate-600 hover:bg-sky-900 hover:shadow-2xl">
              <div class="table-cell border-r border-inherit p-2">ML115</div>
              <div class="table-cell border-r border-inherit p-2">Page 014</div>
              <div class="table-cell border-r border-inherit p-2">Ps 32:4 - Ps 34:7</div>
              <div class="table-cell border-r border-inherit p-2"><span class="pl-12 font-extrabold mr-2">1</span>already done</div>
              <div class="table-cell border-r border-inherit p-2 font-bold">already started</div>
              <div class="table-cell border-r border-inherit p-2"></div>
            </div>
            <div class="table-row border-b border-slate-600 py-3 text-xl shadow-sky-600 last:border-b-0 odd:bg-slate-800 even:bg-slate-600 hover:bg-sky-900 hover:shadow-2xl">
              <div class="table-cell border-r border-inherit p-2">ML115</div>
              <div class="table-cell border-r border-inherit p-2">Page 014</div>
              <div class="table-cell border-r border-inherit p-2">Ps 32:4 - Ps 34:7</div>
              <div class="table-cell border-r border-inherit p-2"><span class="pl-12 mr-2 font-bold">0</span>done</div>
              <div class="table-cell border-r border-inherit p-2 font-bold">already started</div>
              <div class="table-cell border-r border-inherit p-2 font-bold">already published</div>
            </div>
            <div class="table-row border-b border-slate-600 py-3 text-xl shadow-sky-600 last:border-b-0 odd:bg-slate-800 even:bg-slate-600 hover:bg-sky-900 hover:shadow-2xl">
              <div class="table-cell border-r border-inherit p-2">ML115</div>
              <div class="table-cell border-r border-inherit p-2">Page 014</div>
              <div class="table-cell border-r border-inherit p-2">Ps 32:4 - Ps 34:7</div>
              <div class="table-cell border-r border-inherit p-2 text-green-500">
                <div class="inline-flex translate-y-2">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8 translate-x-1">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
                  </svg>
                  <span class="pr-2 pl-4 font-extrabold">2</span> already done
                </div>
              </div>
              <div class="table-cell border-r border-inherit p-2 font-bold"></div>
              <div class="table-cell border-r border-inherit p-2"></div>
            </div>
            <div class="table-row border-b border-slate-600 py-3 text-xl shadow-sky-600 last:border-b-0 odd:bg-slate-800 even:bg-slate-600 hover:bg-sky-900 hover:shadow-2xl">
              <div class="table-cell border-r border-inherit p-2">ML115</div>
              <div class="table-cell border-r border-inherit p-2">Page 014</div>
              <div class="table-cell border-r border-inherit p-2">Ps 32:4 - Ps 34:7</div>
              <div class="table-cell border-r border-inherit p-2 text-green-500">
                <div class="inline-flex translate-y-2">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8 translate-x-1">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
                  </svg>
                  <span class="pr-2 pl-4 font-extrabold">2</span> already done
                </div>
              </div>
              <div class="table-cell border-r border-inherit p-2 font-bold">already started</div>
              <div class="table-cell border-r border-inherit p-2"></div>
            </div>
            <div class="table-row border-b border-slate-600 py-3 text-xl shadow-sky-600 last:border-b-0 odd:bg-slate-800 even:bg-slate-600 hover:bg-sky-900 hover:shadow-2xl">
              <div class="table-cell border-r border-inherit p-2">ML115</div>
              <div class="table-cell border-r border-inherit p-2">Page 014</div>
              <div class="table-cell border-r border-inherit p-2">Ps 32:4 - Ps 34:7</div>
              <div class="table-cell border-r border-inherit p-2 text-green-500">
                <div class="inline-flex translate-y-2">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8 translate-x-1">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
                  </svg>
                  <span class="pr-2 pl-4 font-extrabold">2</span> already done
                </div>
              </div>
              <div class="table-cell border-r border-inherit p-2 font-bold">already started</div>
              <div class="table-cell border-r border-inherit p-2"></div>
            </div>
            <div class="table-row border-b border-slate-600 py-3 text-xl shadow-sky-600 last:border-b-0 odd:bg-slate-800 even:bg-slate-600 hover:bg-sky-900 hover:shadow-2xl">
              <div class="table-cell border-r border-inherit p-2">ML115</div>
              <div class="table-cell border-r border-inherit p-2">Page 014</div>
              <div class="table-cell border-r border-inherit p-2">Ps 32:4 - Ps 34:7</div>
              <div class="table-cell border-r border-inherit p-2 text-green-500">
                <div class="inline-flex translate-y-2">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8 translate-x-1">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
                  </svg>
                  <span class="pr-2 pl-4 font-extrabold">2</span> already done
                </div>
              </div>
              <div class="table-cell border-r border-inherit p-2 font-bold">already started</div>
              <div class="table-cell border-r border-inherit p-2"></div>
            </div>
            <div class="table-row border-b border-slate-600 py-3 text-xl shadow-sky-600 last:border-b-0 odd:bg-slate-800 even:bg-slate-600 hover:bg-sky-900 hover:shadow-2xl">
              <div class="table-cell border-r border-inherit p-2">ML115</div>
              <div class="table-cell border-r border-inherit p-2">Page 014</div>
              <div class="table-cell border-r border-inherit p-2">Ps 32:4 - Ps 34:7</div>
              <div class="table-cell border-r border-inherit p-2 text-green-500">
                <div class="inline-flex translate-y-2">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8 translate-x-1">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
                  </svg>
                  <span class="pr-2 pl-4 font-extrabold">2</span> already done
                </div>
              </div>
              <div class="table-cell border-r border-inherit p-2 font-bold">already started</div>
              <div class="table-cell border-r border-inherit p-2"></div>
            </div>
            <div class="table-row border-b border-slate-600 py-3 text-xl shadow-sky-600 last:border-b-0 odd:bg-slate-800 even:bg-slate-600 hover:bg-sky-900 hover:shadow-2xl">
              <div class="table-cell border-r border-inherit p-2">ML115</div>
              <div class="table-cell border-r border-inherit p-2">Page 014</div>
              <div class="table-cell border-r border-inherit p-2">Ps 32:4 - Ps 34:7</div>
              <div class="table-cell border-r border-inherit p-2 text-green-500">
                <div class="inline-flex translate-y-2">
                  <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-8 translate-x-1">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
                  </svg>
                  <span class="pr-2 pl-4 font-extrabold">2</span> already done
                </div>
              </div>
              <div class="table-cell border-r border-inherit p-2 font-bold">already started</div>
              <div class="table-cell border-r border-inherit p-2"></div>
            </div>
          </div>
        </div>
      </div>
    </div>
    }
}
