use std::thread::sleep;
use std::time::Duration;
use serde::Deserialize;
use serde::Serialize;
use serde_wasm_bindgen::to_value;
use web_sys::{EventTarget, HtmlInputElement};
use yew::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Clone,PartialEq,Deserialize)]
pub struct Task{
    pub id:usize,
    pub task: String
}


#[derive(Deserialize,Serialize)]
#[serde(rename_all = "camelCase")]
struct Args<'a> {
    task_to_add: Option<&'a str>,
    id: Option<usize>,
}

#[function_component(App)]
pub fn app() -> Html {
    let user_input = use_state(String::new);
    let search_input = use_state(String::new);
    let tasks: UseStateHandle<Vec<Task>> = use_state(Vec::new);
    let error_message = use_state(String::new);
    let ontask_input = {
        let user_input = user_input.clone();
        Callback::from(move |e: InputEvent| {
            e.prevent_default();
            let target = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            if let Some(input) = input {
                user_input.set(input.value())
            }
        })
    };
    let onsearch_input = {
        let search_input = search_input.clone();
        Callback::from(move |e: InputEvent| {
            e.prevent_default();
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            if let Some(input) = input {
                search_input.set(input.value())
            }
        })
    };

    let cloned_tasks = tasks.clone();
    use_effect_with(tasks.clone(), move |_|{
        wasm_bindgen_futures::spawn_local(async move{
            let response = invoke("get_all_tasks", JsValue::null()).await.as_string().unwrap();
            let parsed_response : Result<Vec<Task>,serde_json::Error> = serde_json::from_str(&response);
            if let Ok(response) = parsed_response{
                cloned_tasks.set(response)
            }
        });
    });

    let search_for_value = |value: &str, list: &Vec<Task>| {
        list.iter().any(|task_extracted| task_extracted.task == value)
    };
    
    let cloned_error_message = error_message.clone();
    let on_task_addition = {
        let user_input = user_input.clone();
        let tasks = tasks.clone();
        Callback::from(move |_| {
            let error_message = cloned_error_message.clone();
            let input = (*user_input).clone();
            if input.trim().is_empty() {
                error_message.set("Empty user input".to_string());
                sleep(Duration::from_secs(10));
                error_message.set(String::new());
            } else if search_for_value(&input, &tasks) {
                error_message.set("Task already exists".to_string());
                sleep(Duration::from_secs(10));
                error_message.set(String::new());
            } else {
                let tasks = tasks.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let args = Args { task_to_add: Some(&input), id: None };
                    let args_value = to_value(&args).unwrap();
                    invoke("add_task_to_db", args_value).await;
                    let get_response = invoke("get_all_tasks", JsValue::null()).await.as_string().unwrap();
                    let parsed_response: Result<Vec<Task>, serde_json::Error> = serde_json::from_str(&get_response);
                    if let Ok(mut response) = parsed_response{
                        response.push(Task {
                            id: response.len(),
                            task: input.clone(),
                        });
                        error_message.set(String::new());
                        tasks.set(response);
                    }
                });
            }
        })
    };
    
    
    
    let cloned_error_message = error_message.clone();
    let on_task_removal = |id: usize| {
        let tasks = tasks.clone();
        let error_message = cloned_error_message.clone();
        Callback::from(move |_| {
            let mut current_tasks = (*tasks).clone();
    
            if let Some(pos) = current_tasks.iter().position(|task| task.id == id) {
                current_tasks.remove(pos);
    
                for (index, task) in current_tasks.iter_mut().enumerate() {
                    task.id = index;
                }
    
                let args = Args { task_to_add: None, id: Some(id) };
                let args_value = to_value(&args).unwrap();
                wasm_bindgen_futures::spawn_local(async move {
                    invoke("remove_task_from_db", args_value).await;
                });
                error_message.set(String::new());
                tasks.set(current_tasks);
            }
        })
    };
    

    let mut filtered_tasks: Vec<_> = (*tasks)
        .iter()
        .filter(|task_filtered| task_filtered.task.to_lowercase().contains(&*search_input.to_lowercase()))
        .collect();

    filtered_tasks.sort_by(|a, b| a.id.cmp(&b.id));

    let rendered_tasks: Vec<Html> = filtered_tasks
        .into_iter()
        .map(|task_mapped| {
            let on_remove = on_task_removal(task_mapped.id);
            html! {
                <div class="contain-tasks">
                    <div class="remove-items border-2 rounded flex items-center m-4 p-1">
                        <span class="id-box m-4 text-neutral-400">{task_mapped.id}</span>
                        <span class="task-text m-4 text-neutral-400">{task_mapped.clone().task}</span>
                        <button
                            class="flex items-center justify-center h-full px-2 py-1 text-xs bg-red-500 text-white rounded hover:bg-red-600 ml-auto"
                            type="button"
                            value="Remove"
                            onclick={on_remove}
                        >
                            {"Remove"}
                        </button>
                    </div>
                </div>
            }
        })
        .collect();

    html! {
        <>
            <style>
            {"
            *{
                font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, Courier New, monospace;
                }
                
                .contain-tasks{
                    margin:10px;
                }
                
                .task-text {
                    display: inline-block;
                    white-space: nowrap;
                    overflow-x: auto;
                    overflow-y: hidden;
                    scrollbar-width: thin;
                    scrollbar-color: #BF4F74 #F5F5F5;
                }
                
                .task-text::-webkit-scrollbar-track{
                    -webkit-box-shadow: inset 0 0 6px rgba(0,0,0,0.3);
                    border-radius: 10px;
                    background-color: #F5F5F5;
                }
                
                .task-text::-webkit-scrollbar{
                    width: 12px;
                    background-color: #F5F5F5;
                }
                
                .task-text::-webkit-scrollbar-thumb{
                    border-radius: 10px;
                    -webkit-box-shadow: inset 0 0 6px rgba(0,0,0,.3);
                    background-color: #D62929;
                }
                
                .container::-webkit-scrollbar {
                    width: 12px;
                }
                
                .container::-webkit-scrollbar-track {
                    -webkit-box-shadow: inset 0 0 6px rgba(0,0,0,0.3);
                    border-radius: 10px;
                    background-color: #F5F5F5;
                }
                
                .container::-webkit-scrollbar-thumb {
                    border-radius: 10px;
                    -webkit-box-shadow: inset 0 0 6px rgba(0,0,0,0.5);
                    background-color: #BF4F74;
                }
                
                .id-box {
                    border-right: 2px solid #BF4F74;
                    height: 50px;
                    width: 50px;
                    display: flex;
                    justify-content: center;
                    align-items: center; 
                    padding: 0;
                    margin: 0;
                    box-sizing: border-box;
                }
                
                .search-box, .task-input {
                    margin: 10px;
                }
                
                .remove-items{
                    border: 2px solid #BF4F74;
                    width:50%;
                    height:50px;
                    margin:20px auto
                }
                
                .container {
                    position: relative;
                    top: 300px;
                    left: 50%;
                    transform: translate(-50%);
                    padding: 10px;
                    width: 800px;
                    height: 530px;
                    overflow-x: hidden;
                    overflow-y: auto;
                    border: 2px solid #BF4F74;
                }
                
                .entry {
                    position: absolute;
                    top: 30%;
                    left: 50%;
                    transform: translate(-50%, -50%);
                    display: flex;
                    gap: 8px;
                }
                .error {
                    position: relative;
                    top:280px;
                    left:45%
                }
                body {
                    background-color: rgb(82, 82, 82);
                }
                
                .user-input, .butt {
                    border: 2px solid #BF4F74;
                }

                "}
            </style>

            <div class="flex justify-end mb-4">
                <input 
                    class="user-input search-box border-2 p-2 rounded bg-neutral-700"
                    type="text" 
                    placeholder="Search for your task" 
                    oninput={onsearch_input.clone()}
                />
            </div>
            <div class="error">
                {(*error_message).clone()}
            </div>
            <div class="entry">
                    <input
                        class="user-input border-2 p-2 rounded bg-neutral-700"
                        type="text"
                        placeholder="Write Down your task"
                        value={(*user_input).clone()}
                        oninput={ontask_input}
                    />
                    <input 
                        class="butt border-2 text-white p-2 rounded cursor-pointer bg-neutral-700"
                        type="button"
                        value="Add"
                        onclick={on_task_addition}
                    />
            </div>
            <div class="container bg-neutral-700">
                { for rendered_tasks }
            </div>
        </>
    }
}
