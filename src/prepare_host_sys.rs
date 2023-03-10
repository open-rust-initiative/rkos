extern crate dagrs;
//extern crate sys_mount;

use dagrs::RunType;
use dagrs::{init_logger, DagEngine, EnvVar, Inputval, Retval, RunScript, TaskTrait, TaskWrapper};
use libparted::{Device, Disk, FileSystemType, Partition, PartitionFlag, PartitionType};
use nix::mount;

#[macro_use]
use goto::gpoint;

use log::{error, info};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use crate::{utils, vars};

// TODO:将所有本阶段合并统一管理

impl PreparingSoftware {
    //目前使用脚本替代该部分，计划弃用或者完全使用rust
    fn preparing_host_software(&self) {
        //        let packages = &vars::HOST_PACKAGES;
        //        let mut cmd: String = vars::BASE_CONFIG.host_install_cmd.clone();
        //        for package in &packages.host_packages {
        //            cmd += &package;
        //            cmd += " ";
        //        }
        //        println!("{}", &cmd);
        //
        //        let output = Command::new("bash")
        //            .arg("-c")
        //            .arg(cmd)
        //            .status()
        //            .expect("failed to execute process");
        //        assert!(output.success());

        let mut link_list = HashMap::new();
        link_list.insert("sh", "bash");
        link_list.insert("yacc", "bison");
        link_list.insert("awk", "gawk");
        fs::remove_file("/usr/bin/sh").unwrap_or_else(|why| {
            println!("! {:?}", why.kind());
        });
        fs::remove_file("/usr/bin/yacc").unwrap_or_else(|why| {
            println!("! {:?}", why.kind());
        });
        fs::remove_file("/usr/bin/awk").unwrap_or_else(|why| {
            println!("! {:?}", why.kind());
        });
        for link in &link_list {
            // TODO: 换成unix::fs::symlink
            let cmd = format!("sudo ln -s /usr/bin/{} /usr/bin/{}", &link.1, &link.0);
            let create_link_res = Command::new("bash")
                .arg("-c")
                .arg(cmd)
                .status()
                .expect("failed to create link");
            if !create_link_res.success() {
                println!("failed to create link");
            }
        }
    }
    fn preparing_base_software(&self) {
        //创建软件列表，并对应软件下载链接
        //创建临时下载目录
        //给下载目录添加合适权限
        //使用wget下载所有软件包
        //可选的检查所有软件包的正确性

        let all_packages = &vars::ALL_PACKAGES.all_packages;
        let patches = &vars::ALL_PACKAGES.package_patches;
        let mut pack_status = HashMap::new();
        for i in all_packages {
            let mut flag = 0;
            gpoint!['begin:
                if flag>=5{
                    flag=0;
                    pack_status.insert(&i.name,false);
                    break 'begin;
                }

                match utils::download("sources".to_owned(), i.url.clone()){
                    Ok(v)=>{
                        match v{
                            true=>{pack_status.insert(&i.name,v);break 'begin},
                            false=>{continue 'begin},
                        }
                    },
                    Err(_e)=>{continue 'begin;}
                }
            ];
            //            info!("{:?}", i);
            //            let output = Command::new("wget")
            //                .arg("-P")
            //                .arg("sources")
            //                .arg(i.url.as_str())
            //                .status()
            //                .expect("wget failed");
            //            let status = output.success();
            //            pack_status.insert(&i.name, status);
        }
        for i in patches {
            let mut flag = 0;
            gpoint!['begin:
                if flag>=5{
                    flag=0;
                    pack_status.insert(&i.name,false);
                    break 'begin;
                }

                match utils::download("sources/base_patches".to_owned(), i.url.clone()){

                    Ok(v)=>{
                        match v{
                            true=>{pack_status.insert(&i.name,v);break 'begin},
                            false=>{continue 'begin},
                        }
                    },
                    Err(_e)=>{continue 'begin;}
                }
            ];
            //let output = Command::new("wget")
            //    .arg("-P")
            //    .arg("sources")
            //    .arg(i.url.as_str())
            //    .status()
            //    .expect("wget failed");
            //let status = output.success();
            //pack_status.insert(&i.name, status);
        }
        for (k, v) in pack_status {
            info!("{} : {}", k, v);
        }
    }
}

pub struct PreparingSoftware {}
impl TaskTrait for PreparingSoftware {
    fn run(&self, _input: Inputval, _env: EnvVar) -> Retval {
        //        self.preparing_host_software();
        self.preparing_base_software();
        //        self.check();

        Retval::new(())
    }
}

pub struct PreparingEnv {}
impl TaskTrait for PreparingEnv {
    fn run(&self, _input: Inputval, _env: EnvVar) -> Retval {
        let script = RunScript::new("other_script/prepare_env.sh", RunType::SH);
        let res = script.exec(None);
        info!("{:?}", res);
        Retval::empty()
    }
}

pub struct PreparingDisk {}
impl TaskTrait for PreparingDisk {
    fn run(&self, _input: Inputval, _env: EnvVar) -> Retval {
        let script = RunScript::new("other_script/create_dirs.sh", RunType::SH);
        let res = script.exec(None);
        info!("{:?}", res);
        Retval::empty()
    }
}

pub struct CheckEnv {}
impl TaskTrait for CheckEnv {
    fn run(&self, _input: Inputval, _env: EnvVar) -> Retval {
        let script = RunScript::new("other_script/version-check.sh", RunType::SH);
        let res = script.exec(None);
        info!("{:?}", res);
        Retval::empty()
    }
}

//pub struct PreparingDisk {}
//impl PreparingDisk {
//    //在挂载的sdb上创建一个grub bios分区，一般1MB
//    //剩余所有空间创建一个ext4分区
//    //开机自动挂载文件是/etc/fstab
//    //在分区上建立文件系统
//    //挂载分区
//
//    fn preparing_new_filesystem(&self, path: PathBuf) {
//        let mut device = Device::new(path).unwrap();
//        //        for mut device in Device::devices(true) {
//        let mut disk = Disk::new(&mut device).unwrap();
//        for mut part in disk.parts() {
//            println!(
//                "{:?} : {:?} : {:?}",
//                part.name(),
//                part.type_get_name(),
//                part.get_path()
//            );
//        }
//        let fs_type = FileSystemType::get("ext4").expect("no systemtype");
//        println!("{:?}", fs_type.name());
//        assert_eq!(1, 0); //TODO:后续部分不能在本机上测试
//        let mut new_part = Partition::new(
//            &disk,
//            PartitionType::PED_PARTITION_LOGICAL,
//            FileSystemType::get("ext4").as_ref(),
//            0,
//            128,
//        )
//        .unwrap();
//        new_part.set_flag(PartitionFlag::PED_PARTITION_BOOT, true);
//        let constraint = new_part.get_geom();
//        let constraint = match constraint.exact() {
//            Some(v) => v,
//            None => panic!("err"),
//        };
//        //{
//        //   Some(v) => v,
//        //  None => panic!("no constraint"),
//        //};
//        disk.add_partition(&mut new_part, &constraint);
//        //       }
//        println!("over");
//    }
//    fn preparing_new_partition(&self) {}
//    fn preparing_new_dirs(&self) {}
//}
//impl TaskTrait for PreparingDisk {
//    fn run(&self, _input: Inputval, _env: EnvVar) -> Retval {
//        // TODO: 创建分区修改好了，最后再测试，目前还是手动执行命令
//        let path: PathBuf = ["/dev"].iter().collect();
//        self.preparing_new_filesystem(path);
//        Retval::new(())
//    }
//}
//
//struct SettingLfsVariable {}
//impl TaskTrait for SettingLfsVariable {
//    //设定LFS环境变量并保证在所有时刻都可用
//    //可以加入/root/.bash_profile和主目录.bash_profile
//    //需要确认/etc/passwd中为每个需要使用LFS变量的用户指定shell为bash
//    //TODO:计划改为软件运行前用户手动设定
//    fn run(&self, _input: Inputval, _env: EnvVar) -> Retval {
//        Retval::new(())
//    }
//}
//
//struct PrepareEnvironment {}
//impl PrepareEnvironment {
//    //创建lfs目录布局
//    //添加lfs用户
//    //配置lfs环境
//    //配置make的线程数
//    //创建挂载点并挂载LFS分区
//    //已经通过脚本设置好了
//}
//impl TaskTrait for PrepareEnvironment {
//    fn run(&self, _input: Inputval, _env: EnvVar) -> Retval {
//        // TODO: 换成目录拼接，按照全局目录配置信息
//        let target_path = Path::new("./");
//        match target_path.exists() {
//            true => {}
//            false => {}
//        }
//        // 给target_path 实现nixpath的trait 然后就可以用mount了
//        Retval::new(())
//    }
//}
