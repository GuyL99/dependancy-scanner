use serde::{Serialize,Deserialize}; 
use serde_json::Value;
use futures::future::{BoxFuture, FutureExt};
use futures::executor::block_on;
use std::env;


#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Dep{
    pub name:String,
    pub version:String,
    pub deps:Vec<Dep>,
}
async fn check_deps(dep:Dep)->BoxFuture<'static, Vec<Dep>>{
    async move {
        let mut deps = check_deps_for_package(&dep.name,&dep.version).await;
        if deps.len()==0{
            vec![]
        }else{
            for mut dep_g in &mut deps{
                let d_deps = block_on(check_deps(dep_g.clone()));
                dep_g.deps = d_deps.await;
            }
            deps
        }
    }.boxed()
}
async fn check_deps_for_package(pack:&str,version:&str)->Vec<Dep>{
    let json:Value = serde_json::from_str(&reqwest::get(&format!("https://registry.npmjs.org/{}/{}",pack,version)).await.unwrap().text().await.unwrap()).unwrap();
    let deps = json["dependencies"].to_string();
    //let dev_deps = json["devDependencies"].to_string();
    let deps2 = deps.replace("{","").replace("}","").replace("\"","");
    let deps3 = deps2.split(',').collect::<Vec<&str>>();
    let mut dep_vec =vec![];
    for dep in deps3{
        let deper = dep.split(':').collect::<Vec<&str>>();
        if deper.len()<2{
            return vec![];
        }
        let dep1 = Dep{name:deper[0].to_string(),version:deper[1].to_string(),deps:vec![]};
        dep_vec.push(dep1);
    }
    dep_vec
}
fn build_tree_for_deps(deps:Vec<Dep>,layer:u8){
    let lay = std::iter::repeat("|").take(layer.into()).collect::<String>();
    for dep in deps{
        println!("{}+{}",lay,dep.name);
        build_tree_for_deps(dep.deps,layer+1);
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args: Vec<String> = env::args().collect();
    if args.len()==0{
        return Ok(());
    }
    let package = &args[0];
    let version = if args.len()==1{
        "latest"
    }else{
        &args[1]
    };
    let mut deps = check_deps_for_package(package,version).await;
    for mut dep in &mut deps{
        let deps1 = check_deps(dep.clone()).await;
        dep.deps = deps1.await;
    }
    println!("deps tree:");
    build_tree_for_deps(deps,0);
    Ok(())
}
