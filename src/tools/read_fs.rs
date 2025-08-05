use google_ai_rs::proto::{FunctionCall, FunctionDeclaration, FunctionResponse};
use google_ai_rs::Schema;
use prost_types::value::Kind;
use prost_types::{Struct, Value};
use std::collections::{BTreeMap, HashMap};

fn respond_error(error: impl ToString) -> Struct {
    Struct {
        fields: BTreeMap::from([
            ("error".to_string(), Value::from(error.to_string()))
        ]),
    }
}

fn respond_result(result: impl ToString) -> Struct {
    Struct {
        fields: BTreeMap::from([
            ("result".to_string(), Value::from(result.to_string()))
        ]),
    }
}

fn read_fs(path: String) -> Result<String, Box<dyn std::error::Error>> {
    std::fs::read_to_string(&path).map_err(|e| e.into())
}

pub fn handle_read_fs(call: FunctionCall) -> FunctionResponse {
    assert_eq!(call.name, "read_fs");

    let Some(args) = call.args.as_ref() else {
        return FunctionResponse{
            id: call.id,
            name: call.name,
            response: Some(respond_error("Argument is none")),
        };
    };

    let Some(path_value) = args.fields.get("path") else {
        return FunctionResponse{
            id: call.id,
            name: call.name,
            response: Some(respond_error("Required argument 'path' is missing")),
        };
    };

    let Some(kind) = &path_value.kind else {
        return FunctionResponse{
            id: call.id,
            name: call.name,
            response: Some(respond_error("Required argument 'path' is null")),
        };
    };


    let path = match kind {
        Kind::StringValue(s) => s,
        _ => {
            return FunctionResponse{
                id: call.id,
                name: call.name,
                response: Some(respond_error("String argument 'path' is not a string")),
            };
        }
    };

    let resp = match read_fs(path.to_string()) {
        Ok(result) => respond_result(result),
        Err(e) => respond_error(e.to_string())
    };

    FunctionResponse{
        id: call.id,
        name: call.name,
        response: Some(resp),
    }
}

pub fn read_fs_decl() -> FunctionDeclaration {
    FunctionDeclaration {
        name: "read_fs".to_string(),
        description: r#"
        Read file on user's filesystem.
        "#
        .to_string(),
        parameters: Some(Schema {
            r#type: 6, /* OBJECT */
            nullable: false,
            properties: HashMap::from([(
                "path".to_string(),
                Schema {
                    r#type: 1, /* STRING */
                    description: "Path of file to read".to_string(),
                    nullable: false,
                    ..Schema::default()
                },
            )]),
            required: vec!["path".to_string()],
            ..Schema::default()
        }),
        response: Some(Schema{
            r#type: 6, /* OBJECT */
            nullable: false,
            properties: HashMap::from([
                ("error".to_string(), Schema{
                    r#type: 1, /* STRING */
                    description: "(Optional) Error during read".to_string(),
                    nullable: false,
                    ..Schema::default()
                }),
                ("result".to_string(), Schema{
                    r#type: 1, /* STRING */
                    description: "(Optional) Content of file".to_string(),
                    nullable: false,
                    ..Schema::default()
                }),
            ]),
            ..Schema::default()
        }),
    }
}
