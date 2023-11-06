
// Jetporch
// Copyright (C) 2023 - Michael DeHaan <michael@michaeldehaan.net> + contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// long with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::playbooks::context::PlaybookContext;
use std::sync::Arc;
use crate::tasks::*;
use std::sync::RwLock;
use crate::inventory::hosts::Host;
use inline_colorization::{color_red,color_blue,color_green,color_cyan,color_reset,color_yellow};
//use std::marker::{Send,Sync};
use crate::connection::command::CommandResult;
use crate::playbooks::traversal::HandlerMode;

// visitor contains various functions that are called from all over the program
// to send feedback to the user and logs

#[derive(PartialEq)]
pub enum CheckMode {
    Yes,
    No
}

pub struct PlaybookVisitor {
    pub check_mode: CheckMode,
}

impl PlaybookVisitor {

    pub fn new(check_mode: CheckMode) -> Self {
        let s = Self {
            check_mode: check_mode
        };
        s
    }

    pub fn is_check_mode(&self) -> bool { 
        return self.check_mode == CheckMode::Yes; 
    }

    pub fn banner(&self) {
        println!("----------------------------------------------------------");
    }

    // used by the echo module
    pub fn debug_host(&self, host: &Arc<RwLock<Host>>, message: &String) {
        println!("{color_cyan}  ..... {} : {}{color_reset}", host.read().unwrap().name, message);
    }

    pub fn on_playbook_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        let ctx = context.read().unwrap();
        let path = ctx.playbook_path.as_ref().unwrap();
        self.banner();
        println!("> playbook start: {}", path)
    }

    pub fn on_play_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        let play = &context.read().unwrap().play;
        self.banner();
        println!("> play: {}", play.as_ref().unwrap());
    }

    pub fn on_role_start(&self, _context: &Arc<RwLock<PlaybookContext>>) {
    }

    pub fn on_role_stop(&self, _context: &Arc<RwLock<PlaybookContext>>) {
    }

    pub fn on_play_stop(&self, context: &Arc<RwLock<PlaybookContext>>, failed: bool) {
        // failed occurs if *ALL* hosts in a play have failed
        let ctx = context.read().unwrap();
        let play_name = ctx.get_play_name();
        if ! failed {
            self.banner();
            println!("> play complete: {}", play_name);
        } else {
            self.banner();
            println!("{color_red}> play failed: {}{color_reset}", play_name);

        }
    }

    pub fn on_exit(&self, context: &Arc<RwLock<PlaybookContext>>) {
        println!("----------------------------------------------------------");
        println!("");
        show_playbook_summary(context);
    }

    pub fn on_task_start(&self, context: &Arc<RwLock<PlaybookContext>>, is_handler: HandlerMode) {
        let context = context.read().unwrap();
        let task = context.task.as_ref().unwrap();
        let role = &context.role;

        let what = match is_handler {
            HandlerMode::NormalTasks => String::from("task"),
            HandlerMode::Handlers    => String::from("handler")
        };

        self.banner();
        if role.is_none() {
            println!("> begin {}: {}", what, task);
        }
        else {
            println!("> ({}) begin {}: {}", role.as_ref().unwrap().name, what, task);
        }
    }

    pub fn on_batch(&self, batch_num: usize, batch_count: usize, batch_size: usize) {
        self.banner();
        println!("> batch {}/{}, {} hosts", batch_num+1, batch_count, batch_size);
    }

    pub fn on_host_task_start(&self, _context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        println!("… {} => running", host2.name);
    }

    pub fn on_notify_handler(&self, host: &Arc<RwLock<Host>>, which_handler: &String) {
        let host2 = host.read().unwrap();
        println!("… {} => notified: {}", host2.name, which_handler);
    }

    pub fn on_host_delegate(&self, host: &Arc<RwLock<Host>>, delegated: &String) {
        let host2 = host.read().unwrap();
        println!("{color_blue}✓ {} => delegating to: {}{color_reset}",  &host2.name, delegated.clone());
    }

    pub fn on_host_task_ok(&self, context: &Arc<RwLock<PlaybookContext>>, task_response: &Arc<TaskResponse>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        let mut context = context.write().unwrap();
        context.increment_attempted_for_host(&host2.name);
        match &task_response.status {
            TaskStatus::IsCreated  =>  {
                println!("{color_blue}✓ {} => created{color_reset}",  &host2.name);
                context.increment_created_for_host(&host2.name);
            },
            TaskStatus::IsRemoved  =>  {
                println!("{color_blue}✓ {} => removed{color_reset}",  &host2.name);
                context.increment_removed_for_host(&host2.name);
            },
            TaskStatus::IsModified =>  {
                let changes2 : Vec<String> = task_response.changes.iter().map(|x| { format!("{:?}", x) }).collect();
                let change_str = changes2.join(",");
                println!("{color_blue}✓ {} => modified ({}){color_reset}", &host2.name, change_str);
                context.increment_modified_for_host(&host2.name);
            },
            TaskStatus::IsExecuted =>  {
                println!("{color_blue}✓ {} => complete{color_reset}", &host2.name);
                context.increment_executed_for_host(&host2.name);
            },
            TaskStatus::IsPassive  =>  {
                // println!("{color_green}! host: {} => ok (no effect) {color_reset}", &host2.name);
                context.increment_passive_for_host(&host2.name);
            }
            TaskStatus::IsMatched  =>  {
                println!("{color_green}✓ {} => matched {color_reset}", &host2.name);
                context.increment_matched_for_host(&host2.name);

            }
            TaskStatus::IsSkipped  =>  {
                println!("{color_yellow}✓ {} => skipped {color_reset}", &host2.name);
                context.increment_skipped_for_host(&host2.name);

            }
            TaskStatus::Failed => {
                println!("{color_yellow}✓ {} => failed (ignored){color_reset}", &host2.name);
            }
            _ => {
                panic!("on host {}, invalid final task return status, FSM should have rejected: {:?}", host2.name, task_response); 
            }
        }
    }

    // the check mode version of on_host_task_ok - different possible states, slightly different output

    pub fn on_host_task_check_ok(&self, context: &Arc<RwLock<PlaybookContext>>, task_response: &Arc<TaskResponse>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        let mut context = context.write().unwrap();
        context.increment_attempted_for_host(&host2.name);
        match &task_response.status {
            TaskStatus::NeedsCreation  =>  {
                println!("{color_blue}✓ {} => would create{color_reset}",  &host2.name);
                context.increment_created_for_host(&host2.name);
            },
            TaskStatus::NeedsRemoval  =>  {
                println!("{color_blue}✓ {} => would remove{color_reset}",  &host2.name);
                context.increment_removed_for_host(&host2.name);
            },
            TaskStatus::NeedsModification =>  {
                let changes2 : Vec<String> = task_response.changes.iter().map(|x| { format!("{:?}", x) }).collect();
                let change_str = changes2.join(",");
                println!("{color_blue}✓ {} => would modify ({}) {color_reset}", &host2.name, change_str);
                context.increment_modified_for_host(&host2.name);
            },
            TaskStatus::NeedsExecution =>  {
                println!("{color_blue}✓ {} => would run{color_reset}", &host2.name);
                context.increment_executed_for_host(&host2.name);
            },
            TaskStatus::IsPassive  =>  {
                context.increment_passive_for_host(&host2.name);
            }
            TaskStatus::IsMatched  =>  {
                println!("{color_green}✓ {} => matched {color_reset}", &host2.name);
                context.increment_matched_for_host(&host2.name);
            }
            TaskStatus::IsSkipped  =>  {
                println!("{color_yellow}✓ {} => skipped {color_reset}", &host2.name);
                context.increment_skipped_for_host(&host2.name);
            }
            TaskStatus::Failed => {
                println!("{color_yellow}✓ {} => failed (ignored){color_reset}", &host2.name);
            }
            _ => {
                panic!("on host {}, invalid check-mode final task return status, FSM should have rejected: {:?}", host2.name, task_response); 
            }
        }
    }

    pub fn on_host_task_retry(&self, _context: &Arc<RwLock<PlaybookContext>>,host: &Arc<RwLock<Host>>, retries: u64, delay: u64) {
        let host2 = host.read().unwrap();
        println!("{color_blue}! {} => retrying ({} retries left) in {} seconds{color_reset}",host2.name,retries,delay);
    }

    pub fn on_host_task_failed(&self, context: &Arc<RwLock<PlaybookContext>>, task_response: &Arc<TaskResponse>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        if task_response.msg.is_some() {
            let msg = &task_response.msg;
            if task_response.command_result.is_some() {
                {
                    let cmd_result = task_response.command_result.as_ref().as_ref().unwrap();
                    let _lock = context.write().unwrap();
                    println!("{color_red}! {} => failed", host2.name);
                    println!("    cmd: {}", cmd_result.cmd);
                    println!("    out: {}", cmd_result.out);
                    println!("    rc: {}{color_reset}", cmd_result.rc);
                }
            } else {
                println!("{color_red}! error: {}: {}{color_reset}", host2.name, msg.as_ref().unwrap());
            }
        } else {
            println!("{color_red}! host failed: {}, {color_reset}", host2.name);
        }

        context.write().unwrap().increment_failed_for_host(&host2.name);
    }

    pub fn on_host_connect_failed(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        context.write().unwrap().increment_failed_for_host(&host2.name);
        println!("{color_red}! connection failed to host: {}{color_reset}", host2.name);
    }

    pub fn get_exit_status(&self, context: &Arc<RwLock<PlaybookContext>>) -> i32 {
        let failed_hosts = context.read().unwrap().get_hosts_failed_count();
        return match failed_hosts {
            0 => 0,
            _ => 1
        };
    }
    
    pub fn on_before_transfer(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>, path: &String) {
        let host2 = host.read().unwrap();
        if context.read().unwrap().verbosity > 0 {
            println!("{color_blue}! {} => transferring to: {}", host2.name, &path.clone());
        }
    }

    pub fn on_command_run(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>, cmd: &String) {
        let host2 = host.read().unwrap();
        if context.read().unwrap().verbosity > 0 {
            println!("{color_blue}! {} => exec: {}", host2.name, &cmd.clone());
        }
    }

    pub fn on_command_ok(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>, result: &Arc<Option<CommandResult>>,) {
        let host2 = host.read().unwrap();
        let cmd_result = result.as_ref().as_ref().expect("missing command result");
        if context.read().unwrap().verbosity > 2 {
            let _ctx2 = context.write().unwrap(); // lock for multi-line output
            println!("{color_blue}! {} ... command ok", host2.name);
            println!("    cmd: {}", cmd_result.cmd);           
            println!("    out: {}", cmd_result.out.clone());
            println!("    rc: {}{color_reset}", cmd_result.rc);
        }
    }

    pub fn on_command_failed(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>, result: &Arc<Option<CommandResult>>,) {
        let host2 = host.read().expect("context read");
        let cmd_result = result.as_ref().as_ref().expect("missing command result");
        if context.read().unwrap().verbosity > 2 {
            let _ctx2 = context.write().unwrap(); // lock for multi-line output
            println!("{color_red}! {} ... command failed", host2.name);
            println!("    cmd: {}", cmd_result.cmd);
            println!("    out: {}", cmd_result.out.clone());
            println!("    rc: {}{color_reset}", cmd_result.rc);
        }
    }

}

pub fn show_playbook_summary(context: &Arc<RwLock<PlaybookContext>>) {

    let ctx = context.read().unwrap();

    let seen_hosts = ctx.get_hosts_seen_count();
    let role_ct = ctx.get_role_count();
    let task_ct = ctx.get_task_count();
    let action_ct = ctx.get_total_attempted_count();
    let created_ct = ctx.get_total_creation_count();
    let created_hosts = ctx.get_hosts_creation_count();
    let modified_ct = ctx.get_total_modified_count();
    let modified_hosts = ctx.get_hosts_modified_count();
    let removed_ct = ctx.get_total_removal_count();
    let removed_hosts = ctx.get_hosts_removal_count();
    let executed_ct = ctx.get_total_executions_count();
    let executed_hosts = ctx.get_hosts_executions_count();
    let passive_ct = ctx.get_total_passive_count();
    let passive_hosts = ctx.get_hosts_passive_count();
    let matched_ct = ctx.get_total_matched_count();
    let matched_hosts = ctx.get_hosts_matched_count();
    let skipped_ct = ctx.get_total_skipped_count();
    let skipped_hosts = ctx.get_hosts_skipped_count();
    let adjusted_ct = ctx.get_total_adjusted_count();
    let adjusted_hosts = ctx.get_hosts_adjusted_count();
    let unchanged_hosts = seen_hosts - adjusted_hosts;
    let unchanged_ct = action_ct - adjusted_ct;
    let failed_ct    = ctx.get_total_failed_count();
    let failed_hosts = ctx.get_hosts_failed_count();

    let summary = match failed_hosts {
        0 => match adjusted_hosts {
            0 => String::from(format!("{color_green}(✓) Perfect. All hosts matched policy.{color_reset}")),
            _ => String::from(format!("{color_blue}(✓) Actions were applied.{color_reset}")),
        },
        _ => String::from(format!("{color_red}(X) Failures have occured.{color_reset}")),
    };

    let mode_table = format!("|:-|:-|:-|\n\
                      | Results | Items | Hosts \n\
                      | --- | --- | --- |\n\
                      | Roles | {role_ct} | |\n\
                      | Tasks | {task_ct} | {seen_hosts}|\n\
                      | --- | --- | --- |\n\
                      | Matched | {matched_ct} | {matched_hosts}\n\
                      | Created | {created_ct} | {created_hosts}\n\
                      | Modified | {modified_ct} | {modified_hosts}\n\
                      | Removed | {removed_ct} | {removed_hosts}\n\
                      | Executed | {executed_ct} | {executed_hosts}\n\
                      | Passive | {passive_ct} | {passive_hosts}\n\
                      | Skipped | {skipped_ct} | {skipped_hosts}\n\
                      | --- | --- | ---\n\
                      | Unchanged | {unchanged_ct} | {unchanged_hosts}\n\
                      | Changed | {adjusted_ct} | {adjusted_hosts}\n\
                      | Failed | {failed_ct} | {failed_hosts}\n\
                      |-|-|-");

    crate::util::terminal::markdown_print(&mode_table);
    println!("{}", format!("\n{summary}"));
    println!("");



}
