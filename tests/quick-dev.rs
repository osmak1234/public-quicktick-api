use anyhow::Result;
use serde_json::json;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub completed: bool,
    pub user_uuid: String,
}

#[tokio::test]
async fn quick_dev() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:8080")?;

    // Create a new user
    hc.do_post(
        "/post/create_user",
        json!({
            "email": "jonhyS@gmail.com",
            "password": "123469",
            "name": "jon",
        }),
    )
    .await?
    .print()
    .await?;

    // Login and get the user_uuid from the response cookie
    hc.do_get("/login/jonhyS@gmail.com/123469")
        .await?
        .print()
        .await?;

    let uuid_str: String = Uuid::new_v4().hyphenated().to_string();
    println!(
        "
>>>>>>>>>>>>>>>>>>>>>>>>>>>
{}
>>>>>>>>>>>>>>>>>>>>>>>>>>>
",
        uuid_str
    );
    // Test create_task_auth success case with the user_uuid cookie
    let create_task_input = json!({
        "name": "taskname",
        "description": "taskdescription",
        "uuid": uuid_str
    });

    hc.do_post("/post/create_task_cauth", create_task_input.clone())
        .await?
        .print()
        .await?;

    let task = hc.do_get("/get/all_user_tasks_cauth").await?;

    task.print().await?;

    let request_body = task.json_body()?;

    let tasks: Vec<Task> = serde_json::from_value(request_body)?;

    let first_task = tasks.first().unwrap();

    println!(">>> Task uuid: {}", first_task.uuid);
    // this seems to not work because of the library, here is curl command to test it
    //NOTE:
    //   ```bash
    //   curl -X PATCH -H "Content-Type: application/json" -d '{"action": "ToggleCompleted", "task_uuid": ""}' -b "user_uuid=" http://localhost:8080/patch/task
    //   ```
    //
    //   hc.do_patch(
    //       "/patch/task",
    //       json!({
    //           "action": "ToggleCompleted",
    //           "task_uuid": first_task.id
    //       }),
    //   )
    //   .await?
    //   .print()
    //   .await?;

    hc.do_get("/get/all_user_tasks_cauth")
        .await?
        .print()
        .await?;
    // delete user
    hc.do_delete("/delete/user_cauth").await?.print().await?;

    Ok(())
}
