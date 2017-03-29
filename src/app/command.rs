//! Defines command line interface.

use std::io::BufRead;
use std::marker::PhantomData;
use clap;
use shlex;
use core::workspace::Workspace;


pub trait ClapApp {
  fn make_app<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b>;
}

pub fn get_matches<'a, T: ClapApp>() -> clap::ArgMatches<'a> {
  let app = {
    let app = app_from_crate!()
      .setting(clap::AppSettings::VersionlessSubcommands)
      .setting(clap::AppSettings::SubcommandRequiredElseHelp)
      .subcommand(clap::SubCommand::with_name("completion")
                    .about("Generate completion scripts for your shell")
                    .setting(clap::AppSettings::ArgRequiredElseHelp)
                    .arg(clap::Arg::with_name("shell")
                           .help("target shell")
                           .possible_values(&["bash", "zsh", "fish", "powershell"])
                           .required(true))
                    .arg(clap::Arg::from_usage("[out-file]  'path to output script'")));
    T::make_app(app)
  };

  let matches = app.clone().get_matches();
  if let ("completion", Some(m)) = matches.subcommand() {
    let shell =
      m.value_of("shell").and_then(|s| s.parse().ok()).expect("failed to parse target shell");

    if let Some(path) = m.value_of("out-file") {
      let mut file = ::std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(false)
        .open(path)
        .unwrap();
      app.clone().gen_completions_to(env!("CARGO_PKG_NAME"), shell, &mut file);
    } else {
      app.clone().gen_completions_to(env!("CARGO_PKG_NAME"), shell, &mut ::std::io::stdout());
    }
    ::std::process::exit(0);
  }
  matches
}


/// Toplevel application
pub enum Command<'a> {
  New(NewCommand<'a>),
  Clone(CloneCommand<'a>),
  Scan(ScanCommand<'a>),
  List(ListCommand<'a>),
  Foreach(ForeachCommand<'a>),
}

impl<'a> ClapApp for Command<'a> {
  fn make_app<'b, 'c: 'b>(app: clap::App<'b, 'c>) -> clap::App<'b, 'c> {
    app.subcommand(NewCommand::make_app(clap::SubCommand::with_name("new")))
      .subcommand(CloneCommand::make_app(clap::SubCommand::with_name("clone")))
      .subcommand(ListCommand::make_app(clap::SubCommand::with_name("list")))
      .subcommand(ForeachCommand::make_app(clap::SubCommand::with_name("foreach")))
      .subcommand(ScanCommand::make_app(clap::SubCommand::with_name("scan")))
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for Command<'a> {
  fn from(m: &'b clap::ArgMatches<'a>) -> Command<'a> {
    match m.subcommand() {
      ("new", Some(m)) => Command::New(m.into()),
      ("clone", Some(m)) => Command::Clone(m.into()),
      ("list", Some(m)) => Command::List(m.into()),
      ("foreach", Some(m)) => Command::Foreach(m.into()),
      ("scan", Some(m)) => Command::Scan(m.into()),
      _ => unreachable!(),
    }
  }
}

impl<'a> Command<'a> {
  pub fn run(self) -> ::Result<()> {
    match self {
      Command::New(m) => m.run(),
      Command::Clone(m) => m.run(),
      Command::List(m) => m.run(),
      Command::Foreach(m) => m.run(),
      Command::Scan(m) => m.run(),
    }
  }
}


/// Subcommand `new`
pub struct NewCommand<'a> {
  query: &'a str,
  root: Option<&'a str>,
  dry_run: bool,
  ssh: bool,
}

impl<'a> ClapApp for NewCommand<'a> {
  fn make_app<'b, 'c: 'b>(app: clap::App<'b, 'c>) -> clap::App<'b, 'c> {
    app.about("Create a new Git repository with intuitive directory structure")
      .arg_from_usage("<query>         'URL or query of remote repository'")
      .arg_from_usage("--root=[root]   'Target root directory of repository")
      .arg_from_usage("-n, --dry-run   'Do not actually create a new repository'")
      .arg_from_usage("-s, --ssh       'Use SSH protocol'")
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for NewCommand<'a> {
  fn from(m: &'b clap::ArgMatches<'a>) -> NewCommand<'a> {
    NewCommand {
      query: m.value_of("query").unwrap(),
      root: m.value_of("root"),
      dry_run: m.is_present("dry-run"),
      ssh: m.is_present("ssh"),
    }
  }
}

impl<'a> NewCommand<'a> {
  fn run(self) -> ::Result<()> {
    let mut workspace = Workspace::new()?;
    workspace.set_dry_run(self.dry_run);
    if let Some(root) = self.root {
      workspace.set_root(root);
    }

    let query = self.query.parse()?;
    workspace.add_new_repository(query, self.ssh)
  }
}


/// Subcommand `clone`
pub struct CloneCommand<'a> {
  query: Option<&'a str>,
  arg: Option<&'a str>,
  root: Option<&'a str>,
  dry_run: bool,
  ssh: bool,
}

impl<'a> ClapApp for CloneCommand<'a> {
  fn make_app<'b, 'c: 'b>(app: clap::App<'b, 'c>) -> clap::App<'b, 'c> {
    app.about("Clone remote repositories into the root directory")
      .arg_from_usage("[query]         'URL or query of remote repository'")
      .arg_from_usage("--root=[root]   'Target root directory of cloned repository'")
      .arg_from_usage("--arg=[arg]     'Supplemental arguments for Git command'")
      .arg_from_usage("-n, --dry-run   'Do not actually execute Git command'")
      .arg_from_usage("-s, --ssh       'Use SSH protocol'")
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for CloneCommand<'a> {
  fn from(m: &'b clap::ArgMatches<'a>) -> CloneCommand<'a> {
    CloneCommand {
      query: m.value_of("query"),
      arg: m.value_of("arg"),
      root: m.value_of("root"),
      dry_run: m.is_present("dry-run"),
      ssh: m.is_present("ssh"),
    }
  }
}

impl<'a> CloneCommand<'a> {
  fn run(self) -> ::Result<()> {
    let args = self.arg.and_then(|s| shlex::split(s)).unwrap_or_default();

    let mut workspace = Workspace::new()?;
    workspace.set_dry_run(self.dry_run);
    workspace.set_clone_args(args);
    if let Some(root) = self.root {
      workspace.set_root(root);
    }

    if let Some(query) = self.query {
      let query = query.parse()?;
      workspace.clone_repository(query, self.ssh)?;
    } else {
      let stdin = ::std::io::stdin();
      let queries = stdin.lock().lines().filter_map(|l| l.ok());

      for query in queries {
        let query = query.parse()?;
        workspace.clone_repository(query, self.ssh)?;
      }
    }
    Ok(())
  }
}


/// Subcommand `scan`
pub struct ScanCommand<'a> {
  verbose: bool,
  marker: PhantomData<&'a usize>,
}

impl<'a> ClapApp for ScanCommand<'a> {
  fn make_app<'b, 'c: 'b>(app: clap::App<'b, 'c>) -> clap::App<'b, 'c> {
    app.about("Scan directories to create cache of repositories list")
      .arg_from_usage("-v, --verbose  'Use verbose output'")
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for ScanCommand<'a> {
  fn from(m: &'b clap::ArgMatches<'a>) -> ScanCommand<'a> {
    ScanCommand {
      verbose: m.is_present("verbose"),
      marker: PhantomData,
    }
  }
}

impl<'a> ScanCommand<'a> {
  fn run(self) -> ::Result<()> {
    let mut workspace = Workspace::new()?;
    workspace.refresh_cache(self.verbose)?;
    Ok(())
  }
}


/// Subcommand `list`
pub struct ListCommand<'a> {
  marker: PhantomData<&'a usize>,
}

impl<'a> ClapApp for ListCommand<'a> {
  fn make_app<'b, 'c: 'b>(app: clap::App<'b, 'c>) -> clap::App<'b, 'c> {
    app.about("List local repositories managed by rhq")
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for ListCommand<'a> {
  fn from(_: &'b clap::ArgMatches<'a>) -> ListCommand<'a> {
    ListCommand { marker: PhantomData }
  }
}

impl<'a> ListCommand<'a> {
  fn run(self) -> ::Result<()> {
    let mut workspace = Workspace::new()?;
    for repo in workspace.repositories()? {
      println!("{}", repo.path_string());
    }

    Ok(())
  }
}


/// Subcommand `foreach`
pub struct ForeachCommand<'a> {
  command: &'a str,
  args: Option<clap::Values<'a>>,
  dry_run: bool,
}

impl<'a> ClapApp for ForeachCommand<'a> {
  fn make_app<'b, 'c: 'b>(app: clap::App<'b, 'c>) -> clap::App<'b, 'c> {
    app.about("Execute command into each repositories")
      .arg_from_usage("<command>       'Command name'")
      .arg_from_usage("[args]...       'Arguments of command'")
      .arg_from_usage("-n, --dry-run   'Do not actually execute command'")
  }
}

impl<'a, 'b: 'a> From<&'b clap::ArgMatches<'a>> for ForeachCommand<'a> {
  fn from(m: &'b clap::ArgMatches<'a>) -> ForeachCommand<'a> {
    ForeachCommand {
      command: m.value_of("command").unwrap(),
      args: m.values_of("args"),
      dry_run: m.is_present("dry-run"),
    }
  }
}

impl<'a> ForeachCommand<'a> {
  fn run(self) -> ::Result<()> {
    let args: Vec<_> = self.args.map(|s| s.collect()).unwrap_or_default();

    let mut workspace = Workspace::new()?;
    for mut repo in workspace.repositories()? {
      repo.set_dry_run(self.dry_run);
      repo.run_command(self.command, &args)?;
    }

    Ok(())
  }
}
