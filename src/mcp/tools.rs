use std::collections::HashMap;

use crate::{Error, ParamType, Result, Tool, Tools};

impl TryFrom<rmcp::model::Tool> for Tool {
    type Error = Error;

    fn try_from(value: rmcp::model::Tool) -> Result<Self, Self::Error> {
        let schema = value.input_schema;

        let properties = schema
            .get("properties")
            .map(serde_json::to_string)
            .ok_or(Error::tool_schema("Missing properties key"))?
            .and_then(|s| serde_json::from_str::<HashMap<String, ParamType>>(&s))
            .map_err(Error::serde)?;

        let required = schema
            .get("required")
            .map(serde_json::to_string)
            .ok_or(Error::tool_schema("Missing required key"))?
            .and_then(|s| serde_json::from_str::<Vec<String>>(&s))
            .map_err(Error::serde)?;

        let tool = Tool::function(value.name)
            .maybe_description(value.description)
            .with_parameters(|p| p.properties(properties).required(required))
            .build();

        Ok(tool)
    }
}

impl TryFrom<rmcp::model::ListToolsResult> for Tools {
    type Error = Error;

    fn try_from(value: rmcp::model::ListToolsResult) -> Result<Self, Self::Error> {
        Ok(Tools::new(
            value
                .tools
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_>>()?,
        ))
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[test]
    fn deserialize_mcp_tool_json() {
        let target = json!({
          "name": "git_status",
          "description": "Shows the working tree status",
          "inputSchema": {
            "properties": {
              "repo_path": {
                "title": "Repo Path",
                "type": "string"
              }
            },
            "required": [
              "repo_path"
            ],
            "title": "GitStatus",
            "type": "object"
          }
        });
        println!("{:?}", &target);

        let mcp_tool: rmcp::model::Tool = serde_json::from_value(target).unwrap();
        println!("{:?}", &mcp_tool);

        let tool: Tool = mcp_tool.try_into().unwrap();
        println!("{:?}", &tool);
    }

    #[test]
    fn deserialize_mcp_tool_list_result() {
        let target = json!({
          "tools": [
            {
              "name": "git_status",
              "description": "Shows the working tree status",
              "inputSchema": {
                "properties": {
                  "repo_path": {
                    "title": "Repo Path",
                    "type": "string"
                  }
                },
                "required": [
                  "repo_path"
                ],
                "title": "GitStatus",
                "type": "object"
              }
            },
            {
              "name": "git_diff_unstaged",
              "description": "Shows changes in the working directory that are not yet staged",
              "inputSchema": {
                "properties": {
                  "context_lines": {
                    "default": 3,
                    "title": "Context Lines",
                    "type": "integer"
                  },
                  "repo_path": {
                    "title": "Repo Path",
                    "type": "string"
                  }
                },
                "required": [
                  "repo_path"
                ],
                "title": "GitDiffUnstaged",
                "type": "object"
              }
            },
            {
              "name": "git_diff_staged",
              "description": "Shows changes that are staged for commit",
              "inputSchema": {
                "properties": {
                  "context_lines": {
                    "default": 3,
                    "title": "Context Lines",
                    "type": "integer"
                  },
                  "repo_path": {
                    "title": "Repo Path",
                    "type": "string"
                  }
                },
                "required": [
                  "repo_path"
                ],
                "title": "GitDiffStaged",
                "type": "object"
              }
            },
            {
              "name": "git_diff",
              "description": "Shows differences between branches or commits",
              "inputSchema": {
                "properties": {
                  "context_lines": {
                    "default": 3,
                    "title": "Context Lines",
                    "type": "integer"
                  },
                  "repo_path": {
                    "title": "Repo Path",
                    "type": "string"
                  },
                  "target": {
                    "title": "Target",
                    "type": "string"
                  }
                },
                "required": [
                  "repo_path",
                  "target"
                ],
                "title": "GitDiff",
                "type": "object"
              }
            },
            {
              "name": "git_commit",
              "description": "Records changes to the repository",
              "inputSchema": {
                "properties": {
                  "message": {
                    "title": "Message",
                    "type": "string"
                  },
                  "repo_path": {
                    "title": "Repo Path",
                    "type": "string"
                  }
                },
                "required": [
                  "repo_path",
                  "message"
                ],
                "title": "GitCommit",
                "type": "object"
              }
            },
            {
              "name": "git_add",
              "description": "Adds file contents to the staging area",
              "inputSchema": {
                "properties": {
                  "files": {
                    "items": {
                      "type": "string"
                    },
                    "title": "Files",
                    "type": "array"
                  },
                  "repo_path": {
                    "title": "Repo Path",
                    "type": "string"
                  }
                },
                "required": [
                  "repo_path",
                  "files"
                ],
                "title": "GitAdd",
                "type": "object"
              }
            },
            {
              "name": "git_reset",
              "description": "Unstages all staged changes",
              "inputSchema": {
                "properties": {
                  "repo_path": {
                    "title": "Repo Path",
                    "type": "string"
                  }
                },
                "required": [
                  "repo_path"
                ],
                "title": "GitReset",
                "type": "object"
              }
            },
            {
              "name": "git_log",
              "description": "Shows the commit logs",
              "inputSchema": {
                "properties": {
                  "max_count": {
                    "default": 10,
                    "title": "Max Count",
                    "type": "integer"
                  },
                  "repo_path": {
                    "title": "Repo Path",
                    "type": "string"
                  }
                },
                "required": [
                  "repo_path"
                ],
                "title": "GitLog",
                "type": "object"
              }
            },
            {
              "name": "git_create_branch",
              "description": "Creates a new branch from an optional base branch",
              "inputSchema": {
                "properties": {
                  "base_branch": {
                    "anyOf": [
                      {
                        "type": "string"
                      },
                      {
                        "type": "null"
                      }
                    ],
                    "default": null,
                    "title": "Base Branch"
                  },
                  "branch_name": {
                    "title": "Branch Name",
                    "type": "string"
                  },
                  "repo_path": {
                    "title": "Repo Path",
                    "type": "string"
                  }
                },
                "required": [
                  "repo_path",
                  "branch_name"
                ],
                "title": "GitCreateBranch",
                "type": "object"
              }
            },
            {
              "name": "git_checkout",
              "description": "Switches branches",
              "inputSchema": {
                "properties": {
                  "branch_name": {
                    "title": "Branch Name",
                    "type": "string"
                  },
                  "repo_path": {
                    "title": "Repo Path",
                    "type": "string"
                  }
                },
                "required": [
                  "repo_path",
                  "branch_name"
                ],
                "title": "GitCheckout",
                "type": "object"
              }
            },
            {
              "name": "git_show",
              "description": "Shows the contents of a commit",
              "inputSchema": {
                "properties": {
                  "repo_path": {
                    "title": "Repo Path",
                    "type": "string"
                  },
                  "revision": {
                    "title": "Revision",
                    "type": "string"
                  }
                },
                "required": [
                  "repo_path",
                  "revision"
                ],
                "title": "GitShow",
                "type": "object"
              }
            },
            {
              "name": "git_init",
              "description": "Initialize a new Git repository",
              "inputSchema": {
                "properties": {
                  "repo_path": {
                    "title": "Repo Path",
                    "type": "string"
                  }
                },
                "required": [
                  "repo_path"
                ],
                "title": "GitInit",
                "type": "object"
              }
            },
            {
              "name": "git_branch",
              "description": "List Git branches",
              "inputSchema": {
                "properties": {
                  "branch_type": {
                    "description": "Whether to list local branches ('local'), remote branches ('remote') or all branches('all').",
                    "title": "Branch Type",
                    "type": "string"
                  },
                  "contains": {
                    "anyOf": [
                      {
                        "type": "string"
                      },
                      {
                        "type": "null"
                      }
                    ],
                    "default": null,
                    "description": "The commit sha that branch should contain. Do not pass anything to this param if no commit sha is specified",
                    "title": "Contains"
                  },
                  "not_contains": {
                    "anyOf": [
                      {
                        "type": "string"
                      },
                      {
                        "type": "null"
                      }
                    ],
                    "default": null,
                    "description": "The commit sha that branch should NOT contain. Do not pass anything to this param if no commit sha is specified",
                    "title": "Not Contains"
                  },
                  "repo_path": {
                    "description": "The path to the Git repository.",
                    "title": "Repo Path",
                    "type": "string"
                  }
                },
                "required": [
                  "repo_path",
                  "branch_type"
                ],
                "title": "GitBranch",
                "type": "object"
              }
            }
          ]
        });

        let mcp_tool: rmcp::model::ListToolsResult = serde_json::from_value(target).unwrap();

        let tool: Tools = mcp_tool.try_into().unwrap();
        println!("{:?}", &tool);
    }
}
