//! Metadata is evidence about a managed method, not proof of its native ABI.
//! Profiles must be reviewed, compiled in and matched to the actual binary and
//! forwarding thunk. No setting or HTTP parameter can create a trusted profile.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HookTarget {
    pub id: &'static str,
    pub assembly: &'static str,
    pub namespace: &'static str,
    pub class_name: &'static str,
    pub method: &'static str,
    pub expected_arity: u32,
}

pub const TARGETS: &[HookTarget] = &[
    HookTarget { id:"training.success",assembly:"umamusume.dll",namespace:"Gallop",class_name:"SingleModeMainTrainingCuttController",method:"OnSuccessSendCommand",expected_arity:3 },
    HookTarget { id:"training.exec",assembly:"umamusume.dll",namespace:"Gallop",class_name:"SingleModeTrainingCommandService",method:"ExecTraining",expected_arity:2 },
    HookTarget { id:"training.failure_rate",assembly:"umamusume.dll",namespace:"Gallop",class_name:"SingleModeTrainingFailureRateService",method:"GetTrainingFailureRateIgnoreCharaEffect",expected_arity:2 },
    HookTarget { id:"event.add_btn",assembly:"umamusume.dll",namespace:"Gallop",class_name:"StoryChoiceController",method:"AddChoiceButton",expected_arity:1 },
    HookTarget { id:"event.choice",assembly:"umamusume.dll",namespace:"Gallop",class_name:"StoryChoiceController",method:"Choice",expected_arity:2 },
    HookTarget { id:"event.story_set",assembly:"umamusume.dll",namespace:"Gallop",class_name:"StoryManager",method:"SetStory",expected_arity:4 },
    HookTarget { id:"sniff.unity_send",assembly:"UnityEngine.UnityWebRequestModule.dll",namespace:"UnityEngine.Networking",class_name:"UnityWebRequest",method:"SendWebRequest",expected_arity:0 },
    HookTarget { id:"sniff.unity_complete",assembly:"UnityEngine.CoreModule.dll",namespace:"UnityEngine",class_name:"AsyncOperation",method:"InvokeCompletionEvent",expected_arity:0 },
    HookTarget { id:"crypto.md5",assembly:"umamusume.dll",namespace:"Gallop",class_name:"Cryptographer",method:"MakeMd5",expected_arity:1 },
    HookTarget { id:"crypto.hash",assembly:"umamusume.dll",namespace:"Gallop",class_name:"Cryptographer",method:"ComputeHash",expected_arity:1 },
    HookTarget { id:"sniff.compress",assembly:"umamusume.dll",namespace:"Gallop",class_name:"HttpHelper",method:"CompressRequest",expected_arity:1 },
    HookTarget { id:"sniff.decompress",assembly:"umamusume.dll",namespace:"Gallop",class_name:"HttpHelper",method:"DecompressResponse",expected_arity:1 },
    HookTarget { id:"sniff.post",assembly:"Cute.Http.Assembly.dll",namespace:"Cute.Http",class_name:"WWWRequest",method:"Post",expected_arity:3 },
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassIdentity {
    pub assembly: String,
    pub namespace: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MethodDescriptor {
    pub requested: HookTarget,
    pub actual_class: Option<ClassIdentity>,
    pub method_name: Option<String>,
    pub flags: Option<u32>,
    pub implementation_flags: Option<u32>,
    pub is_static: Option<bool>,
    pub parameter_count: Option<u32>,
    pub parameter_types: Vec<Option<String>>,
    pub return_type: Option<String>,
    pub is_generic: Option<bool>,
    pub is_inflated: Option<bool>,
    pub native_address: Option<usize>,
    pub errors: Vec<String>,
}

impl MethodDescriptor {
    pub fn unavailable(requested: HookTarget, error: &str) -> Self {
        Self { requested, actual_class:None, method_name:None, flags:None, implementation_flags:None,
            is_static:None, parameter_count:None, parameter_types:Vec::new(),return_type:None,
            is_generic:None,is_inflated:None,native_address:None,
            errors:if error.is_empty(){Vec::new()}else{vec![error.to_string()]} }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildIdentity {
    pub game_version: String,
    pub unity_version: String,
    pub host_revision: String,
    pub game_binary_sha256: String,
    pub architecture: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgumentLayout {
    InstanceThenManagedArgsThenMethodInfo,
    ManagedArgsThenMethodInfo,
}

#[derive(Clone, Debug)]
pub struct VerifiedProfile {
    pub id: String,
    pub hook_id: String,
    pub build: BuildIdentity,
    pub class: ClassIdentity,
    pub method_name: String,
    pub flags: u32,
    pub implementation_flags: u32,
    pub is_static: bool,
    pub parameter_types: Vec<String>,
    pub return_type: String,
    pub argument_layout: ArgumentLayout,
    pub thunk_id: String,
    /// Digest of reviewed version-specific disassembly/signature/forwarding evidence.
    pub evidence_sha256: String,
}

/// Intentionally empty: no target-game native ABI has been validated for this candidate.
pub fn verified_profiles() -> &'static [VerifiedProfile] { &[] }

/// Current callbacks do not implement a verified MethodInfo-preserving thunk.
pub fn implemented_thunk(_hook_id: &str) -> Option<&'static str> { None }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    ProfileMissing,
    InvalidEvidence,
    BuildUnknown,
    BuildMismatch,
    MetadataIncomplete,
    SignatureMismatch,
    UnsupportedGeneric,
    NativeThunkMissing,
}

impl Rejection {
    pub fn code(self) -> &'static str {
        match self {
            Self::ProfileMissing => "unsupported_native_abi",
            Self::InvalidEvidence => "invalid_abi_evidence",
            Self::BuildUnknown => "unknown_target_build",
            Self::BuildMismatch => "target_build_mismatch",
            Self::MetadataIncomplete => "incomplete_method_metadata",
            Self::SignatureMismatch => "managed_signature_mismatch",
            Self::UnsupportedGeneric => "generic_method_unsupported",
            Self::NativeThunkMissing => "verified_native_thunk_missing",
        }
    }
}

fn sha256_shape(value: &str) -> bool {
    value.len()==64 && value.bytes().all(|v|v.is_ascii_hexdigit())
}

pub fn authorize(method: &MethodDescriptor, observed_build: Option<&BuildIdentity>,
                 implemented_thunk: Option<&str>, profile: Option<&VerifiedProfile>) -> Result<(),Rejection> {
    let profile=profile.ok_or(Rejection::ProfileMissing)?;
    if profile.id.is_empty() || !sha256_shape(&profile.evidence_sha256) || profile.thunk_id.is_empty() {
        return Err(Rejection::InvalidEvidence);
    }
    let build=observed_build.ok_or(Rejection::BuildUnknown)?;
    if build.game_version.is_empty() || build.unity_version.is_empty() || build.host_revision.is_empty()
        || !sha256_shape(&build.game_binary_sha256) || build.architecture.is_empty() {
        return Err(Rejection::BuildUnknown);
    }
    if build!=&profile.build { return Err(Rejection::BuildMismatch); }
    if !method.errors.is_empty() || method.actual_class.is_none() || method.method_name.is_none()
        || method.flags.is_none() || method.implementation_flags.is_none() || method.is_static.is_none()
        || method.return_type.is_none() || method.is_generic.is_none() || method.is_inflated.is_none()
        || method.native_address.is_none_or(|address|address==0)
        || method.parameter_count.is_none_or(|n|n as usize!=method.parameter_types.len())
        || method.parameter_types.iter().any(Option::is_none) {
        return Err(Rejection::MetadataIncomplete);
    }
    if method.is_generic==Some(true) || method.is_inflated==Some(true) {return Err(Rejection::UnsupportedGeneric);}
    if method.requested.id!=profile.hook_id || method.actual_class.as_ref()!=Some(&profile.class)
        || method.method_name.as_ref()!=Some(&profile.method_name)
        || method.flags!=Some(profile.flags) || method.implementation_flags!=Some(profile.implementation_flags)
        || method.is_static!=Some(profile.is_static) || method.return_type.as_ref()!=Some(&profile.return_type)
        || method.parameter_types.iter().map(|v|v.as_deref().unwrap()).ne(profile.parameter_types.iter().map(String::as_str))
        || profile.is_static != matches!(profile.argument_layout,ArgumentLayout::ManagedArgsThenMethodInfo) {
        return Err(Rejection::SignatureMismatch);
    }
    if implemented_thunk!=Some(profile.thunk_id.as_str()) {return Err(Rejection::NativeThunkMissing);}
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (MethodDescriptor,VerifiedProfile) {
        let target=TARGETS[8];
        let class=ClassIdentity{assembly:target.assembly.into(),namespace:target.namespace.into(),name:target.class_name.into()};
        let method=MethodDescriptor{requested:target,actual_class:Some(class.clone()),method_name:Some(target.method.into()),
            flags:Some(0x16),implementation_flags:Some(0),is_static:Some(true),parameter_count:Some(1),
            parameter_types:vec![Some("System.String".into())],return_type:Some("System.String".into()),
            is_generic:Some(false),is_inflated:Some(false),native_address:Some(0x1000),errors:Vec::new()};
        let profile=VerifiedProfile{id:"synthetic-test-only".into(),hook_id:target.id.into(),
            build:BuildIdentity{game_version:"test-game".into(),unity_version:"test-unity".into(),host_revision:"test-host".into(),game_binary_sha256:"a".repeat(64),architecture:"arm64".into()},
            class,method_name:target.method.into(),flags:0x16,implementation_flags:0,is_static:true,
            parameter_types:vec!["System.String".into()],return_type:"System.String".into(),
            argument_layout:ArgumentLayout::ManagedArgsThenMethodInfo,thunk_id:"test-forwarder".into(),evidence_sha256:"b".repeat(64)};
        (method,profile)
    }
    #[test]fn production_candidate_has_no_verified_profile_or_forwarder(){
        assert!(verified_profiles().is_empty());
        for target in TARGETS {assert!(implemented_thunk(target.id).is_none());}
        let (method,profile)=fixture();
        assert_eq!(authorize(&method,Some(&profile.build),Some("test-forwarder"),None),Err(Rejection::ProfileMissing));
    }
    #[test]fn complete_synthetic_contract_matches_but_never_enters_production_allowlist(){
        let (method,profile)=fixture();
        assert_eq!(authorize(&method,Some(&profile.build),Some("test-forwarder"),Some(&profile)),Ok(()));
        assert_eq!(authorize(&method,Some(&profile.build),None,Some(&profile)),Err(Rejection::NativeThunkMissing));
    }
    #[test]fn absent_or_changed_build_and_unreviewed_evidence_are_rejected(){
        let (method,mut profile)=fixture();
        assert_eq!(authorize(&method,None,Some("test-forwarder"),Some(&profile)),Err(Rejection::BuildUnknown));
        let mut changed=profile.build.clone();changed.unity_version="other".into();
        assert_eq!(authorize(&method,Some(&changed),Some("test-forwarder"),Some(&profile)),Err(Rejection::BuildMismatch));
        profile.evidence_sha256.clear();
        assert_eq!(authorize(&method,Some(&profile.build),Some("test-forwarder"),Some(&profile)),Err(Rejection::InvalidEvidence));
    }
    #[test]fn unknown_metadata_and_generic_methods_are_rejected(){
        let (mut method,profile)=fixture();method.is_static=None;
        assert_eq!(authorize(&method,Some(&profile.build),Some("test-forwarder"),Some(&profile)),Err(Rejection::MetadataIncomplete));
        method.is_static=Some(true);method.is_generic=Some(true);
        assert_eq!(authorize(&method,Some(&profile.build),Some("test-forwarder"),Some(&profile)),Err(Rejection::UnsupportedGeneric));
    }
    #[test]fn return_type_parameter_type_static_and_hidden_layout_mismatches_are_rejected(){
        let (method,profile)=fixture();
        for case in 0..4 {
            let mut actual=method.clone();let mut required=profile.clone();
            match case{0=>actual.return_type=Some("System.Byte[]".into()),1=>actual.parameter_types[0]=Some("System.Int32".into()),
                2=>actual.is_static=Some(false),_=>required.argument_layout=ArgumentLayout::InstanceThenManagedArgsThenMethodInfo}
            assert_eq!(authorize(&actual,Some(&required.build),Some("test-forwarder"),Some(&required)),Err(Rejection::SignatureMismatch));
        }
    }
}
