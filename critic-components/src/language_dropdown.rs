use leptos::prelude::*;

use critic_shared::LanguageMetadata;

/// Dropdown list of all known languages available.
#[component]
pub fn LanguageDropDown(
    /// The list of languages that exist
    language_list: impl IntoIterator<Item = LanguageMetadata> + Send + 'static,
    /// Writes the selected value into (and reads from) `selected_language`
    selected_language: RwSignal<Option<i64>>,
    /// the id and name of the select element
    name: &'static str,
    #[prop(optional)] default_string: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="flex justify-center">
            <select
                id=name
                name=name
                class="rounded-md border border-slate-500 text-xl bg-slate-900"
                // when no language is chosen (None), we want to write
                // the empty string into the value here
                prop:value=move || {
                    selected_language.get().map(|m| format!("{m}")).unwrap_or_default()
                }
                on:change:target=move |evt| {
                    selected_language.set(evt.target().value().parse::<i64>().ok());
                }
            >
                <option value="">{default_string.unwrap_or_else(|| "No default Language")}</option>
                {language_list
                    .into_iter()
                    .map(|language| {
                        view! { <option value=language.id>{language.name}</option> }
                    })
                    .collect_view()}
            </select>
        </div>
    }
}
