use i3ipc::reply::{Output, Node};
use i3ipc::I3Connection;
use std::collections::HashMap;
use std::env;
use std::{thread, time};

fn get_monitor_map(conn: &mut I3Connection) -> HashMap<String, u32> {
    let mut outputs = HashMap::new();

    let mut t: Vec<Output> = conn
        .get_outputs().unwrap().outputs
        .into_iter()
        .filter(|x| x.active == true)
        .collect();

    t.sort_by(|a, b| {
        if a.rect.0 != b.rect.0 {
            return a.rect.0.cmp(&b.rect.0);
        }

        return a.rect.1.cmp(&b.rect.1);
    });

    for (index, o) in t.into_iter().enumerate() {
        outputs.insert(o.name.to_owned(), (index+1) as u32);
    }

    outputs
}

fn create_workspaces(conn: &mut I3Connection, num: u32) -> String {
    let outputs = get_monitor_map(conn);

    let workspaces = conn.get_workspaces().unwrap().workspaces;
    let active = workspaces.iter().filter(|x| x.focused == true).nth(0).unwrap();
    let mut current_num : u32 = 0;

    let split = active.name.split(".").collect::<Vec<&str>>();
    if split.len() > 1 {
        current_num = split[1].parse().unwrap();
    } else {
        current_num = outputs.get(&active.output).unwrap().clone();
    }

    for (output, sub_num) in &outputs {
        let new_workspace = format!("{}.{}", num, sub_num);

        conn.run_command(format!("focus output {}", output).as_ref());
        conn.run_command(format!("focus parent").as_ref());
        conn.run_command(format!("workspace --no-auto-back-and-forth {}", new_workspace).as_ref());

        // Sleep time, otherwise workspace creation might fail
        let sleep_time = time::Duration::from_millis(20);
        thread::sleep(sleep_time);
    }

    format!("{}.{}", num, current_num)
}

fn find_active_node(node: &Node) -> Option<&Node> {
    for n in &node.nodes {
        if n.focused == true {
            return Some(n);
        }
    }

    for n in &node.nodes {
        if n.nodes.len() > 0 {
            let active = find_active_node(n);

            if active.is_some() {
                return active;
            }
        }
    }

    None
}

fn get_active_container_id(conn: &mut I3Connection) -> Option<i64> {
    let tree = conn.get_tree().unwrap();

    println!("{:?}", tree);

    for node in &(tree.nodes) {
        let active = find_active_node(node);

        if active.is_some() {
            println!("{}", node.id);
            let node = active.unwrap();
            return Some(node.id);
        }
    }

    None
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            println!("No arguments passed, aborting!");
        },
        2 => {
            println!("Pass a workspace number to switch or move to");
        },
        3 => {
            match args[1].as_str() {
                "switch" => {
                    let workspace: u32 = args[2].parse().expect("Supplied argument was not a number");
                    let mut conn = i3ipc::I3Connection::connect().unwrap();
                    let last_active = create_workspaces(&mut conn, workspace);

                    // Sleep, otherwise re-focusing will not work
                    let sleep_time = time::Duration::from_millis(100);
                    thread::sleep(sleep_time);

                    conn.run_command(format!("workspace {}", last_active).as_ref());
                },
                "move" => {
                    let workspace: u32 = args[2].parse().expect("Supplied argument was not a number");
                    let mut conn = i3ipc::I3Connection::connect().expect("Could not connect to i3ipc");

                    let active_container = get_active_container_id(&mut conn).expect("Could not get active container id");
                    let new_active = create_workspaces(&mut conn, workspace);

                    conn.run_command(format!("[con_id=\"{}\"] move container to workspace {}", active_container, new_active).as_ref());

                    // Sleep, otherwise re-focusing will not work
                    let sleep_time = time::Duration::from_millis(100);
                    thread::sleep(sleep_time);

                    conn.run_command(format!("workspace {}", new_active).as_ref());
                },
                _ => {
                    println!("Invalid argument");
                },
            }

        },
        _ => {
            println!("Wrong number of arguments passed!");
        }
    }
}
