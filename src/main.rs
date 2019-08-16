extern crate notify_rust;
#[macro_use]
extern crate clap;

use clap::{App, AppSettings, Arg, SubCommand};
use notify_rust::hints;
use notify_rust::Notification;

arg_enum!{
pub enum NotificationUrgency{Low, Normal, Critical}
}

fn parse_hint(pattern: &str) {
    let parts = pattern.split(':').collect::<Vec<&str>>();
    assert_eq!(parts.len(), 3);
    println!("{:?}", parts);
    let (_typ, name, value) = (parts[0], parts[1], parts[2]);
    let hint = hints::hint_from_key_val(name, value).unwrap();
    println!("{:?}", hint);
}

fn main() {
    let urgencies = ["low", "normal", "high"];

    let matches = App::new("toastify")
                        .version(&crate_version!()[..])
                        .author("Hendrik Sollich <hendrik@hoodie.de>")
                        .about("sending desktop notifications since 2015")
                        .setting(AppSettings::ArgRequiredElseHelp)
                        .subcommand(SubCommand::with_name("send")
                                    .about("Shows a notification")
                                    // {{{

                                    .arg( Arg::with_name("summary")
                                          .help("Title of the Notification.")
                                          .required(true))

                                    .arg( Arg::with_name("body")
                                          .help("Message body"))

                                    .arg( Arg::with_name("app-name")
                                          .help("Set a specific app-name manually.")
                                          .short("a")
                                          .long("app-name")
                                          .takes_value(true))

                                    .arg( Arg::with_name("expire-time")
                                          .help("Time until expiration in milliseconds. 0 means forever. ")
                                          .short("t")
                                          .long("expire-time")
                                          .takes_value(true))

                                    .arg( Arg::with_name("icon")
                                          .short("i")
                                          .help("Icon of notification.")
                                          .long("icon")
                                          .takes_value(true))

                                    .arg( Arg::with_name("hint")
                                          .help("Specifies basic extra data to pass. Valid types are int, double, string and byte. Pattern: TYPE:NAME:VALUE")
                                          .short("h")
                                          .long("hint")
                                          .takes_value(true))

                                    .arg( Arg::with_name("category")
                                          .help("Set a category.")
                                          .short("c")
                                          .long("category")
                                          .takes_value(true))

                                    .arg( Arg::with_name("urgency")
                                          .help("How urgent is it.")
                                          .short("u")
                                          .long("urgency")
                                          .takes_value(true)
                                          .possible_values(&urgencies))

                                    .arg( Arg::with_name("debug")
                                          .help("Also prints notification to stdout")
                                          .short("d")
                                          .long("debug"))
                                    //}}}
                                    )
                        .subcommand(SubCommand::with_name("info")
                                    .about("Shows information about the running notification server")
                                    )
                        .subcommand(SubCommand::with_name("server")
                                    .about("Starts a little notification server for testing")
                                    )

                        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("server") {
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            use notify_rust::server::NotificationServer;
            use std::thread;
            let server = NotificationServer::create();
            thread::spawn(move || NotificationServer::start(&server, |notification| println!("{:#?}", notification)));

            println!("Press enter to exit.\n");

            std::thread::sleep(std::time::Duration::from_millis(1_000));

            Notification::new()
                .summary("Notification Logger")
                .body("If you can read this in the console, the server works fine.")
                .show()
                .expect("Was not able to send initial test message");

            let mut _devnull = String::new();
            let _ = std::io::stdin().read_line(&mut _devnull);
            println!("Thank you for choosing toastify.");
        }
        #[cfg(target_os = "macos")]
        {
            println!("this feature is not implemented on macOS")
        }
    } else if let Some(_matches) = matches.subcommand_matches("info") {
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            match notify_rust::get_server_information() {
                Ok(info) => println!("server information:\n {:?}\n", info),
                Err(error) => eprintln!("{}", error)
            }

            match notify_rust::get_capabilities() {
                Ok(caps) => println!("capabilities:\n {:?}\n", caps),
                Err(error) => eprintln!("{}", error)
            }
        }
        #[cfg(target_os = "macos")]
        {
            println!("this feature is not implemented on macOS")
        }
    } else if let Some(matches) = matches.subcommand_matches("send") {
        #[cfg(target_os = "macos")]
        {
            // application must be set or mac-notification-sys fails silently
            use notify_rust::{get_bundle_identifier_or_default, set_application};
            let app_id = get_bundle_identifier_or_default("Finder");
            set_application(&app_id).expect("failed to set application");
        }
        let mut notification = Notification::new();

        let summary = matches.value_of("summary").unwrap();
        notification.summary(summary);

        if let Some(appname) = matches.value_of("app-name") {
            notification.appname(appname);
        }

        if let Some(icon) = matches.value_of("icon") {
            notification.icon(icon);
        }

        if let Some(body) = matches.value_of("body") {
            notification.body(body);
        }

        if let Some(categories) = matches.value_of("category") {
            //notification.body(body);
            for category in categories.split(':') {
                notification.hint(notify_rust::NotificationHint::Category(category.to_owned()));
            }
        }

        if let Some(timeout_string) = matches.value_of("expire-time") {
            if let Ok(timeout) = timeout_string.parse::<i32>() {
                notification.timeout(timeout);
            } else {
                println!(
                    "can't parse timeout {:?}, please use a number",
                    timeout_string
                );
            }
        }

        if matches.is_present("urgency") {
            let urgency = value_t_or_exit!(matches.value_of("urgency"), NotificationUrgency);
            // TODO: somebody make this a cast, please!
            match urgency {
                NotificationUrgency::Low => {
                    notification.urgency(notify_rust::NotificationUrgency::Low)
                }
                NotificationUrgency::Normal => {
                    notification.urgency(notify_rust::NotificationUrgency::Normal)
                }
                NotificationUrgency::Critical => {
                    notification.urgency(notify_rust::NotificationUrgency::Critical)
                }
            };
        }

        if let Some(hint) = matches.value_of("hint") {
            println!("{:?}", hint);
            parse_hint(hint);
            std::process::exit(0);
        }

        if matches.is_present("debug") {
            #[cfg(all(unix, not(target_os = "macos")))]
            {
            if let Err(error) = notification.show_debug() {
                eprintln!("{}", error)
            }
            }
            #[cfg(target_os = "macos")]
            {
                println!("this feature is not implemented on macOS")
            }
        } else {
            if let Err(error) = notification.show() {
                eprintln!("{}", error)
            }
        }
    }
}
