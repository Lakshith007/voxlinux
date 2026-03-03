use crate::reader::find_plan;
use voxlinuxd::explain::explain_at_level;

pub fn explain_plan(id: &str, level: u8) {
    if let Some(plan) = find_plan(id) {
        println!("Plan ID      : {}", plan.id);
        println!("Issue        : {}", plan.issue);
        println!("Risk         : {:?}", plan.risk);
        println!("High Conf    : {}", plan.confidence_high);
        println!("Reversible   : {}", plan.reversible);
        println!("Reboot Req   : {}", plan.requires_reboot);

        explain_at_level(&plan.explain, level);
    } else {
        println!("Plan not found.");
    }
}
