use serde::Deserialize;
use uuid::Uuid;
use std::path::PathBuf;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ProjectId(Uuid);

impl ProjectId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Deserialize)]
pub struct ProjectPath {
    project_id: ProjectId,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SupportedUnityVersion {
    R2019_4_31,
    R2022_3_6,
    #[cfg(feature = "sdk-unstable")]
    /// > The Editor may crash when updating a shader graph reference by another shader using UsePass.
    /// > This is an issue with Unity 2022.3.6f1 and is fixed in 2022.3.14f1.
    /// -- https://creators.vrchat.com/releases/release-3-5-0#known-issues
    R2022_3_14,
}

impl SupportedUnityVersion {
    #[inline]
    pub const fn build_hash(self) -> &'static str {
        match self {
            SupportedUnityVersion::R2019_4_31 => "bd5abf232a62",
            SupportedUnityVersion::R2022_3_6  => "b9e6e7e9fa2d",
            #[cfg(feature = "sdk-unstable")]
            SupportedUnityVersion::R2022_3_14 => "eff2de9070d8"
        }
    }

    #[inline]
    pub const fn fully_qualified_version(self) -> &'static str {
        match self {
            SupportedUnityVersion::R2019_4_31 => "2019.4.31f1",
            SupportedUnityVersion::R2022_3_6  => "2022.3.6f1",
            #[cfg(feature = "sdk-unstable")]
            SupportedUnityVersion::R2022_3_14 => "2022.3.14f1",
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NewProjectTemplateKind {
    Avatar3WithUnity2019,
    World3WithUnity2019,
    Avatar3WithUnity2022,
    World3WithUnity2022,
}

impl NewProjectTemplateKind {
    fn unity(self) -> SupportedUnityVersion {
        match self {
            NewProjectTemplateKind::Avatar3WithUnity2019 => SupportedUnityVersion::R2019_4_31,
            NewProjectTemplateKind::World3WithUnity2019 => SupportedUnityVersion::R2019_4_31,
            NewProjectTemplateKind::Avatar3WithUnity2022 => SupportedUnityVersion::R2022_3_6,
            NewProjectTemplateKind::World3WithUnity2022 => SupportedUnityVersion::R2022_3_6,
        }
    }

    fn kind(self) -> ProjectKind {
        match self {
            NewProjectTemplateKind::Avatar3WithUnity2019 => ProjectKind::Avatar3,
            NewProjectTemplateKind::World3WithUnity2019 =>  ProjectKind::World3,
            NewProjectTemplateKind::Avatar3WithUnity2022 => ProjectKind::Avatar3,
            NewProjectTemplateKind::World3WithUnity2022 =>  ProjectKind::World3,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ProjectKind {
    /// avatar project using SDK 3.
    Avatar3,
    /// world project using SDK 3.
    World3,
}

#[derive(Deserialize)]
pub struct NewProjectTemplateRequest {
    template_kind: NewProjectTemplateKind,
    project_root: PathBuf,
}

#[derive(Deserialize)]
pub struct UnityPath {
    pub path: PathBuf
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct UnityInstallSetting {
    pub version: SupportedUnityVersion,
    pub target: UnityPlatformTarget,
    pub host: UnityEditorHost,
    pub prefer_unity_hub: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum UnityPlatformTarget {
    /// Build for windows.
    WindowsMono,
    /// Build for Meta Quest and Android cell-phones.
    Android,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum UnityEditorHost {
    Windows,
    Linux,
    MacOS
}
