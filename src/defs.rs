use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct Struct {
    #[serde(flatten)]
    pub fields: BTreeMap<String, Value>,
}

impl From<prost_types::Struct> for Struct {
    fn from(value: prost_types::Struct) -> Self {
        Self {
            fields: value
                .fields
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

impl Into<prost_types::Struct> for Struct {
    fn into(self) -> prost_types::Struct {
        prost_types::Struct {
            fields: self
                .fields
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct ListValue {
    pub values: Vec<Value>,
}

impl From<prost_types::ListValue> for ListValue {
    fn from(value: prost_types::ListValue) -> Self {
        Self {
            values: value.values.into_iter().map(|v| v.into()).collect(),
        }
    }
}

impl Into<prost_types::ListValue> for ListValue {
    fn into(self) -> prost_types::ListValue {
        prost_types::ListValue {
            values: self.values.into_iter().map(|v| v.into()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Kind {
    NullValue(i32),
    NumberValue(f64),
    StringValue(String),
    BoolValue(bool),
    StructValue(Struct),
    ListValue(ListValue),
}

impl From<prost_types::value::Kind> for Kind {
    fn from(value: prost_types::value::Kind) -> Self {
        match value {
            prost_types::value::Kind::NullValue(v) => Kind::NullValue(v),
            prost_types::value::Kind::NumberValue(v) => Kind::NumberValue(v),
            prost_types::value::Kind::StringValue(v) => Kind::StringValue(v),
            prost_types::value::Kind::BoolValue(v) => Kind::BoolValue(v),
            prost_types::value::Kind::StructValue(v) => Kind::StructValue(v.into()),
            prost_types::value::Kind::ListValue(v) => Kind::ListValue(v.into()),
        }
    }
}

impl Into<prost_types::value::Kind> for Kind {
    fn into(self) -> prost_types::value::Kind {
        match self {
            Kind::NullValue(v) => prost_types::value::Kind::NullValue(v),
            Kind::NumberValue(v) => prost_types::value::Kind::NumberValue(v),
            Kind::StringValue(v) => prost_types::value::Kind::StringValue(v),
            Kind::BoolValue(v) => prost_types::value::Kind::BoolValue(v),
            Kind::StructValue(v) => prost_types::value::Kind::StructValue(v.into()),
            Kind::ListValue(v) => prost_types::value::Kind::ListValue(v.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct Value {
    pub kind: Option<Kind>,
}

impl From<prost_types::Value> for Value {
    fn from(value: prost_types::Value) -> Self {
        Self {
            kind: value.kind.map(|v| v.into()),
        }
    }
}

impl Into<prost_types::Value> for Value {
    fn into(self) -> prost_types::Value {
        prost_types::Value {
            kind: self.kind.map(|v| v.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FunctionCall {
    pub id: String,
    pub name: String,
    pub args: Option<Struct>,
}

impl From<google_ai_rs::FunctionCall> for FunctionCall {
    fn from(value: google_ai_rs::FunctionCall) -> Self {
        Self {
            id: value.id,
            name: value.name,
            args: value.args.map(|v| v.into()),
        }
    }
}

impl Into<google_ai_rs::FunctionCall> for FunctionCall {
    fn into(self) -> google_ai_rs::FunctionCall {
        google_ai_rs::FunctionCall {
            id: self.id,
            name: self.name,
            args: self.args.map(|v| v.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FunctionResponse {
    pub id: String,
    pub name: String,
    pub response: Option<Struct>,
}

impl From<google_ai_rs::proto::FunctionResponse> for FunctionResponse {
    fn from(value: google_ai_rs::proto::FunctionResponse) -> Self {
        Self {
            id: value.id,
            name: value.name,
            response: value.response.map(|v| v.into()),
        }
    }
}

impl Into<google_ai_rs::proto::FunctionResponse> for FunctionResponse {
    fn into(self) -> google_ai_rs::proto::FunctionResponse {
        google_ai_rs::proto::FunctionResponse {
            id: self.id,
            name: self.name,
            response: self.response.map(|v| v.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Blob {
    pub mime_type: String,
    pub data: Vec<u8>,
}

impl From<google_ai_rs::proto::Blob> for Blob {
    fn from(value: google_ai_rs::proto::Blob) -> Self {
        Self {
            mime_type: value.mime_type,
            data: value.data,
        }
    }
}

impl Into<google_ai_rs::proto::Blob> for Blob {
    fn into(self) -> google_ai_rs::proto::Blob {
        google_ai_rs::proto::Blob {
            mime_type: self.mime_type,
            data: self.data,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileData {
    pub mime_type: String,
    pub file_uri: String,
}

impl From<google_ai_rs::proto::FileData> for FileData {
    fn from(value: google_ai_rs::proto::FileData) -> Self {
        Self {
            mime_type: value.mime_type,
            file_uri: value.file_uri,
        }
    }
}

impl Into<google_ai_rs::proto::FileData> for FileData {
    fn into(self) -> google_ai_rs::proto::FileData {
        google_ai_rs::proto::FileData {
            mime_type: self.mime_type,
            file_uri: self.file_uri,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ExecutableCode {
    pub language: i32,
    pub code: String,
}

impl From<google_ai_rs::proto::ExecutableCode> for ExecutableCode {
    fn from(value: google_ai_rs::proto::ExecutableCode) -> Self {
        Self {
            language: value.language,
            code: value.code,
        }
    }
}

impl Into<google_ai_rs::proto::ExecutableCode> for ExecutableCode {
    fn into(self) -> google_ai_rs::proto::ExecutableCode {
        google_ai_rs::proto::ExecutableCode {
            language: self.language,
            code: self.code,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CodeExecutionResult {
    pub outcome: i32,
    pub output: String,
}

impl From<google_ai_rs::proto::CodeExecutionResult> for CodeExecutionResult {
    fn from(value: google_ai_rs::proto::CodeExecutionResult) -> Self {
        Self {
            outcome: value.outcome,
            output: value.output,
        }
    }
}

impl Into<google_ai_rs::proto::CodeExecutionResult> for CodeExecutionResult {
    fn into(self) -> google_ai_rs::proto::CodeExecutionResult {
        google_ai_rs::proto::CodeExecutionResult {
            outcome: self.outcome,
            output: self.output,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Data {
    Text{ text: String },
    InlineData(Blob),
    FunctionCall(FunctionCall),
    FunctionResponse(FunctionResponse),
    FileData(FileData),
    ExecutableCode(ExecutableCode),
    CodeExecutionResult(CodeExecutionResult),
}

impl From<String> for Data {
    fn from(value: String) -> Self {
        Data::Text { text: value }
    }
}

impl From<google_ai_rs::Data> for Data {
    fn from(value: google_ai_rs::Data) -> Self {
        match value {
            google_ai_rs::Data::Text(v) => Self::Text{ text: v },
            google_ai_rs::Data::InlineData(v) => Self::InlineData(v.into()),
            google_ai_rs::Data::FunctionCall(v) => Self::FunctionCall(v.into()),
            google_ai_rs::Data::FunctionResponse(v) => Self::FunctionResponse(v.into()),
            google_ai_rs::Data::FileData(v) => Self::FileData(v.into()),
            google_ai_rs::Data::ExecutableCode(v) => Self::ExecutableCode(v.into()),
            google_ai_rs::Data::CodeExecutionResult(v) => Self::CodeExecutionResult(v.into()),
        }
    }
}

impl Into<google_ai_rs::Data> for Data {
    fn into(self) -> google_ai_rs::Data {
        match self {
            Data::Text{ text } => google_ai_rs::Data::Text(text),
            Data::InlineData(v) => google_ai_rs::Data::InlineData(v.into()),
            Data::FunctionCall(v) => google_ai_rs::Data::FunctionCall(v.into()),
            Data::FunctionResponse(v) => google_ai_rs::Data::FunctionResponse(v.into()),
            Data::FileData(v) => google_ai_rs::Data::FileData(v.into()),
            Data::ExecutableCode(v) => google_ai_rs::Data::ExecutableCode(v.into()),
            Data::CodeExecutionResult(v) => google_ai_rs::Data::CodeExecutionResult(v.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Part {
    #[serde(flatten)]
    pub data: Option<Data>,
}

impl Part {
    pub fn new(data: Data) -> Self {
        Self { data: Some(data) }
    }
}

impl From<google_ai_rs::Part> for Part {
    fn from(value: google_ai_rs::Part) -> Self {
        Self {
            data: value.data.map(|v| v.into()),
        }
    }
}

impl Into<google_ai_rs::Part> for Part {
    fn into(self) -> google_ai_rs::Part {
        google_ai_rs::Part {
            data: self.data.map(|v| v.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Content {
    pub parts: Vec<Part>,
    pub role: String,
}

impl Content {
    pub fn system(parts: Vec<Part>) -> Self {
        Self {
            parts,
            role: "system".to_string(),
        }
    }

    pub fn tool(parts: Vec<Part>) -> Self {
        Self {
            parts,
            role: "tool".to_string(),
        }
    }
}

impl From<google_ai_rs::proto::Content> for Content {
    fn from(value: google_ai_rs::Content) -> Self {
        Self {
            parts: value.parts.into_iter().map(|v| v.into()).collect(),
            role: value.role,
        }
    }
}

impl Into<google_ai_rs::proto::Content> for Content {
    fn into(self) -> google_ai_rs::proto::Content {
        google_ai_rs::proto::Content {
            parts: self.parts.into_iter().map(|v| v.into()).collect(),
            role: self.role,
        }
    }
}
