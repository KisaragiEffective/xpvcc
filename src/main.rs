mod model;

use std::io::{BufWriter, Write};
use std::process::Command;
use std::sync::Mutex;
use actix_web::{App, get, HttpResponseBuilder, HttpServer, post, Responder};
use actix_web::web::{Json, Path, Query};
use once_cell::sync::Lazy;
use rand::random;
use reqwest::{Client, ClientBuilder, StatusCode, Url};
use reqwest::tls::{Version as TlsVersion};
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, System, SystemExt};
use crate::model::{NewProjectTemplateRequest as NewProjectTemplate, ProjectPath, SupportedUnityVersion, UnityEditorHost, UnityInstallSetting, UnityPath};

static HTTP_CLIENT: Lazy<Client> = Lazy::new(||
    ClientBuilder::new()
        .https_only(true)
        .gzip(true)
        .min_tls_version(TlsVersion::TLS_1_2)
        .user_agent("XPVCC/0.1.0 (contact: GitHub.com/KisaragiEffective)")
        .build()
        .expect("failed to initialize HTTP client")
);

static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| {
    let mut s = System::new();
    s.refresh_processes();

    Mutex::new(s)
});

#[post("/project/{project_id}/open")]
async fn open_project(path: Path<ProjectPath>) -> String {
    todo!()
}

#[get("/project/{project_id}/dependency")]
async fn get_dependency_of_project(path: Path<ProjectPath>) -> String {
    todo!()
}

#[post("/project/new/{template_kind}")]
async fn make_new_project(path: Path<NewProjectTemplate>) -> String {
    todo!()
}

#[post("/project/{project_id}/backup")]
async fn create_backup_of_project(path: Path<ProjectPath>) -> String {
    todo!()
}

#[derive(Deserialize)]
struct ConfirmUnityInstallation {
    installed_path: UnityPath,
    completion_token: InstallCompletionToken,
}

#[post("/unity/tell")]
async fn tell_existing_unity_install_location(fs_path: Query<ConfirmUnityInstallation>) -> impl Responder {
    let fs_path = fs_path.into_inner();
    if let InstallCompletionToken::Pid(pid) = fs_path.completion_token {
        SYSTEM.lock().unwrap().refresh_process(pid);

        if let Some(_) = SYSTEM.lock().unwrap().process(pid) {
            return HttpResponseBuilder::new(StatusCode::BAD_REQUEST).message_body(format!("Please exit installer (pid = {pid})"))
        }
    }

    let p = &fs_path.installed_path.path;

    if p.exists() && p.is_dir() {
        // TODO: persistent
        HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).message_body("persistence is not implemented yet")
    } else {
        HttpResponseBuilder::new(StatusCode::BAD_REQUEST).message_body(format!("Installation is not found on {}", p.display()))
    }
}

#[derive(Deserialize, Serialize)]
enum InstallCompletionToken {
    Pid(Pid),
    DelegateToHub(HubDelegateToken),
}

#[derive(Eq, PartialEq, Deserialize, Serialize)]
struct HubDelegateToken([u8; 32]);

impl HubDelegateToken {
    pub fn new() -> Self {
        Self(random())
    }
}

#[derive(Serialize)]
struct AwaitUnityInstall {
    completion_token: InstallCompletionToken,
    message: &'static str
}

#[post("/unity/install")]
async fn install_unity(query: Query<UnityInstallSetting>) -> impl Responder {
    let query = query.into_inner();
    let host = query.host;
    let version = query.version;

    if query.prefer_unity_hub {
        let uri = format!("unityhub://{revision}/{hash}", revision = version.fully_qualified_version(), hash = version.build_hash());

        if let Ok(_) = open::that(uri) {
            // TODO: register delegate token to in-memory table
            Json(AwaitUnityInstall {
                completion_token: InstallCompletionToken::DelegateToHub(HubDelegateToken::new()),
                message: "Please complete installation. `POST /unity/tell` with installed directory afterwards."
            })
        }
    }

    let res = HTTP_CLIENT.get(compute_url(host, version))
        .send().await.expect("failed to establish HTTP request");

    if res.status().is_client_error() {
        return HttpResponseBuilder::new(StatusCode::SERVICE_UNAVAILABLE)
            .content_type("plain/text; charset=utf-8")
            .message_body("failed to download Unity installer");
    }

    let res = res.bytes().await.expect("failed to decode body");

    let temp = tempfile::NamedTempFile::new().expect("failed to create temporary file");
    let installer_temporary = temp.path();
    let mut bw = BufWriter::new(temp);
    bw.write_all(res.as_ref()).expect("failed to eizokuka");

    // headless install support status is vary, so throw them away.
    // Child: !Drop, so it will not be killed even if current execution is outside of it
    let mut installer_ps = Command::new(installer_temporary).spawn().expect("failed to spawn installer");

    Json(AwaitUnityInstall {
        completion_token: InstallCompletionToken::Pid(installer_ps.id().into()),
        message: "Please complete installation. `POST /unity/tell` with installed directory afterwards."
    })
}

struct LocInfo {
    extension: &'static str,
    executable_name: &'static str,
}

fn compute_url(unity_editor_host: UnityEditorHost, version: SupportedUnityVersion) -> Url {
    let mut buf = String::with_capacity(120);
    buf.push_str("https://download.unity3d.com/download_unity/");
    buf.push_str(version.build_hash());
    buf.push('/');
    let executable_name = match unity_editor_host {
        UnityEditorHost::Windows => "UnityDownloadAssistant",
        UnityEditorHost::Linux => "UnitySetup",
        UnityEditorHost::MacOS => "UnityDownloadAssistant"
    };
    buf.push_str(executable_name);
    buf.push('-');
    buf.push_str(version.fully_qualified_version());
    let extension = match unity_editor_host {
        UnityEditorHost::Windows => ".exe",
        UnityEditorHost::Linux => "",
        UnityEditorHost::MacOS => ".dmg"
    };
    buf.push_str(extension);

    Url::parse(&buf).expect("must not fail")
}

// project manage
// - create new from template
// - version management of dependent packages
// make backup
// unity version manage
// - unity install

#[actix_web::main]
async fn main() {
    HttpServer::new(||
        App::new()
            .service(install_unity)
            .service(tell_existing_unity_install_location)
            .service(create_backup_of_project)
            .service(get_dependency_of_project)
            .service(make_new_project)
            .service(open_project)
    )
        .bind(("127.0.0.1", 51901))
        .expect("address is in use")
        .run()
        .await
        .expect("failed in actix_web");
}
