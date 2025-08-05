use glob::glob;
use google_ai_rs::proto::{FunctionDeclaration, FunctionResponse};
use google_ai_rs::{FunctionCall, Schema};
use libc::{S_IFBLK, S_IFCHR, S_IFDIR, S_IFIFO, S_IFLNK, S_IFREG, S_IFSOCK};
use prost_types::value::Kind;
use prost_types::value::Kind::StructValue;
use prost_types::{Struct, Value};
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fs;
use std::os::linux::fs::MetadataExt;
use std::path::PathBuf;

struct FileType(u32);

impl FileType {
    fn is(&self, b: u32) -> bool {
        (self.0 & libc::S_IFMT) == b
    }
}

impl Into<char> for FileType {
    fn into(self) -> char {
        if self.is(S_IFREG) {
            '-'
        } else if self.is(S_IFDIR) {
            'd'
        } else if self.is(S_IFLNK) {
            'l'
        } else if self.is(S_IFCHR) {
            'c'
        } else if self.is(S_IFBLK) {
            'b'
        } else if self.is(S_IFIFO) {
            'p'
        } else if self.is(S_IFSOCK) {
            's'
        } else {
            '?'
        }
    }
}

struct FileEntry {
    path: String,
    uid: u32,
    gid: u32,
    mode: String,
}

impl Into<Struct> for FileEntry {
    fn into(self) -> Struct {
        Struct {
            fields: BTreeMap::from([
                ("path".to_string(), Value::from(self.path)),
                ("uid".to_string(), Value::from(self.uid)),
                ("gid".to_string(), Value::from(self.gid)),
                ("mode".to_string(), Value::from(self.mode)),
            ]),
        }
    }
}

fn mode_to_str(mode: u32) -> String {
    let mut v: [char; 10] = ['-'; 10];

    v[0] = <FileType as Into<char>>::into(FileType(mode));

    let tbl: [char; 9] = ['r', 'w', 'x', 'r', 'w', 'x', 'r', 'w', 'x'];

    // 3-digit oct
    for i in 0..9 {
        let mask = 1 << (8 - i);
        if (mode & mask) != 0 {
            v[i + 1] = tbl[i];
        }
    }

    // 4-digit oct
    if mode & 0b001000000000 != 0 {
        v[8 + 1] = 't';
    }
    if mode & 0b010000000000 != 0 {
        v[5 + 1] = 's';
    }
    if mode & 0b100000000000 != 0 {
        v[2 + 1] = 's';
    }

    v.into_iter().collect()
}

fn path_to_entry(path: PathBuf) -> Result<FileEntry, Box<dyn Error>> {
    let metadata = fs::symlink_metadata(&path)?;

    Ok(FileEntry {
        path: path.to_string_lossy().to_string(),
        uid: metadata.st_uid(),
        gid: metadata.st_gid(),
        mode: mode_to_str(metadata.st_mode()),
    })
}

fn search_fs(pattern: &str) -> (Vec<FileEntry>, Vec<String>) {
    let mut entries: Vec<FileEntry> = vec![];
    let mut errors: Vec<String> = vec![];

    let glob = match glob(pattern) {
        Ok(glob) => glob,
        Err(e) => {
            errors.push(e.to_string());
            return (entries, errors);
        }
    };

    for entry in glob {
        let Ok(path) = entry else {
            continue;
        };

        let entry = match path_to_entry(path) {
            Ok(entry) => entry,
            Err(e) => {
                errors.push(e.to_string());
                continue;
            }
        };

        entries.push(entry);
    }

    (entries, errors)
}

fn respond_error(errors: Vec<String>) -> Struct {
    let errors: Vec<Value> = errors
        .into_iter()
        .map(|s| Value::from(s))
        .collect();

    Struct {
        fields: BTreeMap::from([
            ("results".to_string(), Value::from(vec![])),
            ("errors".to_string(), Value::from(errors))
        ]),
    }
}

fn respond(success: Vec<FileEntry>, errors: Vec<String>) -> Struct {
    let success = success
        .into_iter()
        .map(|entry| <FileEntry as Into<Struct>>::into(entry.into()))
        .map(|s| Value::from(StructValue(s)))
        .collect::<Vec<Value>>();
    let errors = errors
        .into_iter()
        .map(|v| Value::from(v))
        .collect::<Vec<Value>>();

    Struct {
        fields: BTreeMap::from([
            ("results".to_string(), Value::from(success)),
            ("errors".to_string(), Value::from(errors))
        ]),
    }
}

pub fn handle_search_fs(call: FunctionCall) -> FunctionResponse {
    assert_eq!(call.name, "search_fs");

    let Some(args) = call.args.as_ref() else {
        return FunctionResponse{
            id: call.id,
            name: call.name,
            response: Some(respond_error(vec!["Argument is none".to_string()])),
        };
    };

    let Some(pattern_value) = args.fields.get("pattern") else {
        return FunctionResponse{
            id: call.id,
            name: call.name,
            response: Some(respond_error(vec!["Required argument 'pattern' is missing".to_string()])),
        };
    };

    let Some(kind) = &pattern_value.kind else {
        return FunctionResponse{
            id: call.id,
            name: call.name,
            response: Some(respond_error(vec!["Required argument 'pattern' is null".to_string()])),
        };
    };

    let pattern = match kind {
        Kind::StringValue(s) => s,
        _ => {
            return FunctionResponse{
                id: call.id,
                name: call.name,
                response: Some(respond_error(vec!["String argument 'pattern' is not a string".to_string()])),
            };
        }
    };

    let (success, errors) = search_fs(pattern);

    FunctionResponse{
        id: call.id,
        name: call.name,
        response: Some(respond(success, errors)),
    }
}

pub fn search_fs_decl() -> FunctionDeclaration {
    FunctionDeclaration {
        name: "search_fs".to_string(),
        description: r#"
        Search file or directory on user's filesystem using glob expression.
        Error and successful result can be returned at once,
        when if operation failed for only some of files (e.g. insufficient permission)

        ## Usage

        The glob expression syntax is same as standard UNIX glob expression syntax.

        ## Examples

        - `/repos/**/*.cxx` : Find `.cxx` file in `/repos` recursively
        - `/repos/*.h` : Find `.h` file in `/repos` not-recursively

        "#
        .to_string(),
        parameters: Some(Schema {
            r#type: 6, /* OBJECT */
            nullable: false,
            properties: HashMap::from([(
                "pattern".to_string(),
                Schema {
                    r#type: 1, /* STRING */
                    description: "Glob expression to search".to_string(),
                    nullable: false,
                    ..Schema::default()
                },
            )]),
            required: vec!["pattern".to_string()],
            ..Schema::default()
        }),
        response: Some(Schema {
            r#type: 6, /* OBJECT */
            nullable: false,
            properties: HashMap::from([
                (
                    "errors".to_string(),
                    Schema {
                        r#type: 5, /* ARRAY */
                        description: "Exceptions occurred during operation".to_string(),
                        nullable: true,
                        items: Some(Box::new(Schema {
                            r#type: 1, /* STRING */
                            nullable: false,
                            ..Schema::default()
                        })),
                        ..Schema::default()
                    },
                ),
                (
                    "results".to_string(),
                    Schema {
                        r#type: 5, /* ARRAY */
                        description: "An array of glob search result".to_string(),
                        nullable: true,
                        items: Some(Box::new(Schema {
                            r#type: 6, /* OBJECT */
                            description: "A glob search result".to_string(),
                            nullable: false,
                            properties: HashMap::from([
                                (
                                    "path".to_string(),
                                    Schema {
                                        r#type: 1, /* STRING */
                                        nullable: false,
                                        ..Schema::default()
                                    },
                                ),
                                (
                                    "uid".to_string(),
                                    Schema {
                                        r#type: 3, /* INTEGER */
                                        nullable: false,
                                        ..Schema::default()
                                    },
                                ),
                                (
                                    "gid".to_string(),
                                    Schema {
                                        r#type: 3, /* INTEGER */
                                        nullable: false,
                                        ..Schema::default()
                                    },
                                ),
                                (
                                    "mode".to_string(),
                                    Schema {
                                        r#type: 1, /* STRING */
                                        nullable: false,
                                        ..Schema::default()
                                    },
                                ),
                            ]),
                            required: vec![
                                "path".to_string(),
                                "uid".to_string(),
                                "gid".to_string(),
                                "mode".to_string(),
                            ],
                            ..Schema::default()
                        })),
                        max_items: i64::MAX,
                        min_items: 0,
                        ..Schema::default()
                    },
                ),
            ]),
            required: vec![],
            ..Schema::default()
        }),
    }
}
