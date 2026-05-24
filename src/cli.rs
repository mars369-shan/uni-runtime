use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Parser)]
pub enum Commands {
    /// 创建新环境
    Create {
        /// 环境名称
        #[arg(value_name = "ENV_NAME")]
        name: String,
        /// 发行版类型
        #[arg(short, long, value_name = "DISTRO")]
        distro: String,
    },
    /// 列出所有环境
    List,
    /// 删除环境
    Delete {
        /// 环境名称
        #[arg(value_name = "ENV_NAME")]
        name: String,
    },
    /// 启动环境
    Start {
        /// 环境名称
        #[arg(value_name = "ENV_NAME")]
        name: String,
    },
    /// 停止环境
    Stop {
        /// 环境名称
        #[arg(value_name = "ENV_NAME")]
        name: String,
    },
    /// 查看运行中的环境
    Ps,
    /// 设置默认环境
    #[command(name = "set")]
    SetDefault {
        /// 环境名称
        #[arg(value_name = "ENV_NAME")]
        name: Option<String>,
        /// 创建并设置默认环境
        #[arg(long, value_name = "DISTRO")]
        create: Option<String>,
    },
    /// 取消默认环境设置
    #[command(name = "unset")]
    UnsetDefault,
    /// 在环境中执行命令
    Exec {
        /// 环境名称 (可选，使用默认环境时省略)
        #[arg(value_name = "ENV_NAME")]
        name: Option<String>,
        /// 要执行的命令
        #[arg(value_name = "COMMAND", trailing_var_arg = true, last = true)]
        command: Vec<String>,
    },
    /// 在环境中运行交互式 shell
    Run {
        /// 环境名称 (可选，使用默认环境时省略)
        #[arg(value_name = "ENV_NAME")]
        name: Option<String>,
    },
    /// 创建快照
    Snapshot {
        /// 环境名称
        #[arg(value_name = "ENV_NAME")]
        name: String,
        /// 快照名称
        #[arg(short, long, value_name = "NAME")]
        snapshot_name: String,
    },
    /// 恢复快照
    Restore {
        /// 环境名称
        #[arg(value_name = "ENV_NAME")]
        name: String,
        /// 快照名称
        #[arg(value_name = "SNAPSHOT_NAME")]
        snapshot_name: String,
    },
    /// 查看环境详情
    Info {
        /// 环境名称 (默认环境时使用 'default')
        #[arg(value_name = "ENV_NAME")]
        name: String,
    },
    /// 运行功能测试
    Test,
}
