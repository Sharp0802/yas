use crate::defs::*;
use crate::tools::{handle_read_fs, handle_search_fs};
use crate::MODEL;
use bytes::Bytes;
use hyper::body::Frame;
use lazy_static::lazy_static;
use serde::Serialize;
use std::convert::Infallible;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

lazy_static! {
    static ref HISTORY: Mutex<Vec<Content>> = Mutex::new(vec![]);
}

fn frame_from_json<T: Serialize>(v: &T) -> Frame<Bytes> {
    let json = serde_json::to_string(v).unwrap();
    let sse_event = format!("data: {}\n\n", json);
    Frame::data(Bytes::from(sse_event))
}

pub async fn get_chat() -> Vec<Content> {
    HISTORY.lock().await.clone()
}

pub async fn add_chat(chat: Content) {
    HISTORY.lock().await.push(chat);
}

async fn process_chat_once(sender: &Sender<Result<Frame<Bytes>, Infallible>>) -> bool {
    let mut history = HISTORY.lock().await;

    let contents_copy = history
        .iter()
        .cloned()
        .map(Into::into)
        .collect::<Vec<google_ai_rs::Content>>();

    let mut response_stream = match MODEL
        .get()
        .unwrap()
        .stream_generate_content(contents_copy)
        .await {
        Ok(stream) => stream,
        Err(e) => {
            let chat = Content::system(vec![
                Part::new(Data::from(format!("Error while generating stream content: {:?}", e)))
            ]);
            let _ = sender.send(Ok(frame_from_json(&chat))).await;
            return false;
        }
    };

    let mut function_called = false;

    while let Some(resp) = match response_stream.next().await {
        Ok(part) => part,
        Err(e) => {
            let chat = Content::system(vec![
                Part::new(Data::from(format!("Error while iterating stream: {:?}", e)))
            ]);
            let _ = sender.send(Ok(frame_from_json(&chat))).await;
            return false;
        }
    } {
        let Some(candidate) = resp.candidates.first() else {
            continue;
        };

        if candidate.finish_reason != /* STOP */ 1 && candidate.finish_reason != /* NONE */ 0 {
            let chat = Content::system(vec![
                Part::new(Data::from(format!("Generation failed with code: {:}", candidate.finish_reason)))
            ]);
            let _ = sender.send(Ok(frame_from_json(&chat))).await;
            return false;
        }

        let Some(content) = &candidate.content else {
            continue;
        };
        let content: Content = content.clone().into();

        history.push(content.clone().into());

        let _ = sender.send(Ok(frame_from_json(&content))).await;

        let mut function_responses: Vec<Part> = Vec::new();

        for part in content.parts {
            let Some(data) = part.data else {
                continue;
            };

            if let Data::FunctionCall(call) = data {
                function_called = true;

                match handle_function_call(call).await {
                    Ok(resp) => {
                        function_responses.push(Part::new(Data::FunctionResponse(resp)))
                    }
                    Err(e) => {
                        function_responses.push(Part::new(Data::from(e)))
                    }
                };
            }
        }

        if !function_responses.is_empty() {
            let function_response_content = Content::tool(function_responses);
            let _ = sender.send(Ok(frame_from_json(&function_response_content))).await;
            history.push(function_response_content);
        }
    }

    function_called
}

async fn handle_function_call(call: FunctionCall) -> Result<FunctionResponse, String> {
    match call.name.as_str() {
        "search_fs" => Ok(handle_search_fs(call.into()).into()),
        "read_fs" => Ok(handle_read_fs(call.into()).into()),
        _ => Err(format!("Unknown function '{}'", call.name)),
    }
}

pub async fn process_chat(sender: Sender<Result<Frame<Bytes>, Infallible>>) {
    while process_chat_once(&sender).await {
    }
}
