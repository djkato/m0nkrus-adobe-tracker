#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::fs;

// hide console window on Windows in release
use downloader::Downloader;
use eframe::egui;
use eframe::{run_native, App, NativeOptions};
use regex::Regex;
use version_compare::Version;
use walkdir::WalkDir;

fn main() -> Result<(), eframe::Error> {
    let mut installed_apps = find_local_programs();
    /*let mut installed_apps: Vec<LocalFoundApp> = vec![
        LocalFoundApp {
            name: "Adobe After Effects".to_owned(),
            version: "1.0.0".to_owned(),
            newest_online: None,
        },
        LocalFoundApp {
            name: "Adobe Premiere Pro".to_owned(),
            version: "1.0.0".to_owned(),
            newest_online: None,
        },
        LocalFoundApp {
            name: "Adobe Photoshop".to_owned(),
            version: "1.0.0".to_owned(),
            newest_online: None,
        },
    ];*/
    let mut online_apps = find_online_programs(&installed_apps);
    compare_versions(&mut installed_apps, &mut online_apps);
    create_ui(installed_apps, online_apps)
}

fn create_ui(
    installed_apps: Vec<LocalFoundApp>,
    online_apps: Vec<OnlineFoundApp>,
) -> Result<(), eframe::Error> {
    //egui
    let options = NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    let app = Box::new(IsaApp {
        local_app_list: installed_apps,
        online_app_list: online_apps,
    });
    run_native("Adobe checker", options, Box::new(|_cc| app))
}
/* GUI */
#[derive(Default)]
struct IsaApp {
    local_app_list: Vec<LocalFoundApp>,
    online_app_list: Vec<OnlineFoundApp>,
}
impl App for IsaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Updates");
            ui.separator();
            for local_app in self.local_app_list.iter() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", &local_app.name));
                    if local_app.newest_online.is_some() {
                        ui.style_mut().visuals.hyperlink_color = egui::Color32::RED;
                        ui.hyperlink_to(
                            format!(
                                "Found newer version! Current: {}, Newest: {}",
                                local_app.version,
                                local_app.newest_online.as_ref().unwrap().version.clone()
                            ),
                            local_app.newest_online.as_ref().unwrap().magnet.clone(),
                        );
                        ui.reset_style();
                    } else {
                        ui.style_mut().visuals.override_text_color = Some(egui::Color32::GREEN);
                        ui.label(format!(
                            "Version Up to date! Current:{}",
                            &local_app.version
                        ));
                        ui.reset_style();
                    }
                });
            }
            ui.add_space(20.0);
            ui.heading("Online found apps");
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for online_app in self.online_app_list.iter() {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{}, version: {} ",
                            online_app.name, &online_app.version
                        ));
                        ui.hyperlink_to("download", &online_app.magnet);
                        ui.add_space(ui.available_width());
                    });
                }
            });
        });
    }
}

/* COMPARE VERS */
fn compare_versions(
    installed_apps: &mut Vec<LocalFoundApp>,
    online_apps: &mut Vec<OnlineFoundApp>,
) {
    for local_app in installed_apps.iter_mut() {
        for online_app in online_apps.iter() {
            if online_app.name.contains(&local_app.name) {
                if local_app.newest_online.is_none() {
                    if Version::from(&local_app.version) < Version::from(&online_app.version) {
                        local_app.newest_online = Some(online_app.clone());
                    }
                } else {
                    if Version::from(&local_app.newest_online.as_ref().unwrap().version)
                        < Version::from(&online_app.version)
                    {
                        local_app.newest_online = Some(online_app.clone());
                    }
                }
            }
        }
    }
}

/* SCAPER */
fn find_online_programs(app_list: &Vec<LocalFoundApp>) -> Vec<OnlineFoundApp> {
    let version_brackets_regex = Regex::new(r"\(v.*?\)").unwrap();
    let version_bare_regex = Regex::new(r"v[.\d]* ").unwrap();
    let magnet_regex = Regex::new(r#"href="magnet:\?xt.*?""#).unwrap();
    let name_regex = Regex::new(r#"<b>Adobe .*<wbr>"#).unwrap();
    //if temp is missing make it, delete previous tracker.php file if there is one
    match std::fs::read_dir("./temp") {
        Ok(_) => std::fs::remove_file("./temp/tracker.php").unwrap_or_else(|_e| ()),
        Err(_) => std::fs::create_dir("./temp").unwrap(),
    }

    //Downloads file
    let mut downloader = Downloader::builder()
        .download_folder(std::path::Path::new("./temp"))
        .build()
        .unwrap();
    let dl = downloader::Download::new("http://rutracker.ru/tracker.php?pid=1334502");

    let result = downloader.download(&[dl]).unwrap();

    //if downloaded, parse site
    let mut online_apps = Vec::new();
    if let Ok(_) = &result[0] {
        println!("");
        let website_file = fs::read_to_string("./temp/tracker.php").unwrap();
        for (web_line_i, web_line) in website_file.lines().enumerate() {
            if web_line.to_ascii_lowercase().contains("adobe") {
                let mut version = "".to_owned();
                let mut name = "".to_owned();
                if let Some(res) = name_regex.find(web_line) {
                    name = web_line
                        .get(res.start() + 3..res.end() - 5)
                        .unwrap()
                        .to_string();
                }
                if let Some(res) = version_brackets_regex.find(web_line) {
                    version = web_line
                        .get(res.start() + 2..res.end() - 1)
                        .unwrap()
                        .to_string();
                    if version.contains("<wbr>") {
                        version.pop();
                        version.pop();
                        version.pop();
                        version.pop();
                        version.pop();
                    }
                } else if let Some(res) = version_bare_regex.find(&web_line) {
                    version = web_line
                        .get(res.start() + 1..res.end() - 1)
                        .unwrap()
                        .to_string();
                    version = version.trim().to_string();
                }

                let mut magnet = "".to_string();
                for magnet_web_line in website_file.lines().skip(web_line_i) {
                    if magnet_web_line.contains("href=\"magnet:?") {
                        if let Some(magnet_res) = magnet_regex.find(magnet_web_line) {
                            magnet = magnet_web_line
                                .get(magnet_res.start() + 6..magnet_res.end() - 1)
                                .unwrap()
                                .to_owned();
                            break;
                        }
                    }
                }
                println!("App: {}\nVersion: {}\nMagnet:{}\n", &name, &version, magnet);
                let mut online_app = OnlineFoundApp {
                    name,
                    magnet,
                    version,
                };
                online_apps.push(online_app.clone());
            }
        }
    };
    online_apps
}

/* FILE BROWSER */
fn find_local_programs() -> Vec<LocalFoundApp> {
    let version_regex = Regex::new(r#""\{\w*-\d*\.\d.*?-64-"#).unwrap();
    let mut apps = Vec::new();
    for directory_res in WalkDir::new(r"C:\Program Files\Adobe").max_depth(1) {
        if let Ok(directory) = directory_res {
            let mut version = "".to_owned();

            //find AMT/application.xml inside app folder & get version
            for files_res in WalkDir::new(directory.path()) {
                if let Ok(files) = files_res {
                    if files.path().ends_with("application.xml") {
                        println!("{}", files.path().as_os_str().to_str().unwrap());
                        let xml_file;
                        xml_file = std::fs::read_to_string(files.path()).unwrap();

                        for (i, line) in xml_file.lines().enumerate() {
                            let xml_res_line: usize = i;
                            let xml_res = version_regex.find(line);
                            if xml_res.is_some() {
                                version = xml_file
                                    .lines()
                                    .nth(xml_res_line)
                                    .unwrap()
                                    .get(xml_res.unwrap().start() + 7..xml_res.unwrap().end() - 4)
                                    .unwrap()
                                    .to_string();
                                break;
                            }
                        }
                        break;
                    }
                }
            }

            if let Some(app_name) = directory.path().file_name() {
                let mut app_name_str: String = app_name.to_str().unwrap().into();
                if let Some(adobe_app_name_usize) = app_name_str.find("2") {
                    app_name_str.truncate(adobe_app_name_usize);
                    app_name_str = app_name_str.trim().to_string();
                    println!("App: {}, Version: {}", &app_name_str, &version);
                    apps.push(LocalFoundApp {
                        version,
                        name: app_name_str,
                        newest_online: None,
                    });
                }
            }
        }
    }
    apps
}

#[derive(Clone)]
pub struct OnlineFoundApp {
    pub version: String,
    pub name: String,
    pub magnet: String,
}

pub struct LocalFoundApp {
    pub version: String,
    pub name: String,
    pub newest_online: Option<OnlineFoundApp>,
}
