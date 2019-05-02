#![cfg_attr(feature = "unstable", feature(specialization))]
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;
use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::WORD;

type CowStr = Cow<'static, str>;
type CowPath = Cow<'static, Path>;

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct Lang(WORD, WORD);

pub mod lang {
    use super::Lang;
    use winapi::shared::ntdef::*;

    pub const LANG_ENU: Lang = Lang(LANG_ENGLISH, SUBLANG_ENGLISH_US);
    pub const PRESET_LANG_1: &[Lang] = &[LANG_ENU];

    pub const LANG_CHS: Lang = Lang(LANG_CHINESE, SUBLANG_CHINESE_SIMPLIFIED);
    pub const LANG_CHT: Lang = Lang(LANG_CHINESE, SUBLANG_CHINESE_TRADITIONAL);
    pub const LANG_DEU: Lang = Lang(LANG_GERMAN, SUBLANG_GERMAN);
    pub const LANG_ESN: Lang = Lang(LANG_SPANISH, SUBLANG_SPANISH);
    pub const LANG_FRA: Lang = Lang(LANG_FRENCH, SUBLANG_FRENCH);
    pub const LANG_ITA: Lang = Lang(LANG_ITALIAN, SUBLANG_ITALIAN);
    pub const LANG_JPN: Lang = Lang(LANG_JAPANESE, SUBLANG_JAPANESE_JAPAN);
    pub const LANG_KOR: Lang = Lang(LANG_KOREAN, SUBLANG_KOREAN);

    pub const PRESET_LANG_9: &[Lang] = &[
        LANG_ENU, LANG_CHS, LANG_CHT, LANG_DEU, LANG_ESN, LANG_FRA, LANG_ITA, LANG_JPN, LANG_KOR,
    ];

    pub const LANG_RUS: Lang = Lang(LANG_RUSSIAN, SUBLANG_RUSSIAN_RUSSIA);

    pub const PRESET_LANG_10: &[Lang] = &[
        LANG_ENU, LANG_CHS, LANG_CHT, LANG_DEU, LANG_ESN, LANG_FRA, LANG_ITA, LANG_JPN, LANG_KOR,
        LANG_RUS,
    ];

    pub const LANG_CSY: Lang = Lang(LANG_CZECH, SUBLANG_CZECH_CZECH_REPUBLIC);
    pub const LANG_PLK: Lang = Lang(LANG_POLISH, SUBLANG_POLISH_POLAND);
    pub const LANG_PTB: Lang = Lang(LANG_PORTUGUESE, SUBLANG_PORTUGUESE_BRAZILIAN);
    pub const LANG_TRK: Lang = Lang(LANG_TURKISH, SUBLANG_TURKISH_TURKEY);

    pub const PRESET_LANG_14: &[Lang] = &[
        LANG_ENU, LANG_CHS, LANG_CHT, LANG_CSY, LANG_DEU, LANG_ESN, LANG_FRA, LANG_ITA, LANG_JPN,
        LANG_KOR, LANG_PLK, LANG_PTB, LANG_RUS, LANG_TRK,
    ];
}

pub struct Build {
    resources: BTreeMap<Lang, Vec<(IdOrName, Box<dyn Resource>)>>,
}

impl Build {
    pub fn new(languages: &[Lang]) -> Self {
        let mut build = Build {
            resources: BTreeMap::new(),
        };
        for language in languages.iter().cloned() {
            build.resources.insert(language, Vec::new());
        }
        build
    }

    pub fn with_one_language() -> Self {
        Self::new(lang::PRESET_LANG_1)
    }

    pub fn with_one_or_two_languages(l: Lang) -> Self {
        if l == lang::LANG_ENU {
            Self::with_one_language()
        } else {
            Self::with_two_languages(l)
        }
    }

    pub fn with_two_languages(l: Lang) -> Self {
        assert_ne!(l, lang::LANG_ENU);
        Self::new(&[lang::LANG_ENU, l])
    }

    pub fn with_nine_languages() -> Self {
        Self::new(lang::PRESET_LANG_9)
    }

    pub fn resource(
        mut self,
        id_or_name: impl Into<IdOrName>,
        resource: impl Resource + Clone,
    ) -> Self {
        let id_or_name: IdOrName = id_or_name.into();
        for (_lang, lang_specific_resources) in self.resources.iter_mut() {
            lang_specific_resources.push((id_or_name.clone(), Box::new(resource.clone())));
        }
        self
    }

    pub fn lang_specific_resource(
        mut self,
        language: Lang,
        id_or_name: impl Into<IdOrName>,
        resource: impl Resource,
    ) -> Self {
        let id_or_name: IdOrName = id_or_name.into();
        let lang_specific_resources = self.resources.entry(language).or_default();
        lang_specific_resources.push((id_or_name, Box::new(resource)));
        self
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Id(WORD);

impl From<WORD> for Id {
    fn from(v: WORD) -> Self {
        Id(v)
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum IdOrName {
    Id(Id),
    Name(CowStr),
}

impl From<WORD> for IdOrName {
    fn from(v: WORD) -> Self {
        IdOrName::Id(Id(v))
    }
}

impl From<String> for IdOrName {
    fn from(v: String) -> Self {
        IdOrName::Name(Cow::Owned(v))
    }
}

#[cfg(not(feature = "unstable"))]
impl From<&'static str> for IdOrName {
    fn from(v: &'static str) -> Self {
        IdOrName::Name(Cow::Borrowed(v))
    }
}

#[cfg(feature = "unstable")]
default impl<'a> From<&'a str> for IdOrName {
    fn from(v: &'a str) -> Self {
        IdOrName::Name(Cow::Owned(v.to_owned()))
    }
}

#[cfg(feature = "unstable")]
default impl<'a> From<&'a str> for IdOrName
where
    'a: 'static,
{
    fn from(v: &'a str) -> Self {
        IdOrName::Name(Cow::Borrowed(v))
    }
}

pub trait Resource: 'static {}

#[macro_use]
pub mod resource {
    use crate::{CowPath, Resource};
    use std::path::Path;
    use std::rc::Rc;

    fn create_path_only_resource_from_file<T, R: From<CowPath>>(
        path: impl AsRef<Path>,
        ty: impl FnOnce(Rc<R>) -> T,
    ) -> T {
        use std::borrow::Cow;
        ty(Rc::new(R::from(Cow::Owned(path.as_ref().to_owned()))))
    }

    macro_rules! define_path_only_resource {
        ($type_name:ident) => {
            #[derive(Clone)]
            pub struct $type_name(Rc<CowPath>);

            impl $type_name {
                pub fn from_file(path: impl AsRef<Path>) -> Self {
                    create_path_only_resource_from_file(path, $type_name)
                }
            }

            impl Resource for $type_name {}
        };
    }

    macro_rules! define_builder_generated_resource {
        ($type_name:ident, $data_type:path, $builder_type:path) => {
            #[derive(Clone)]
            pub struct $type_name(pub(crate) Rc<$data_type>);

            impl $type_name {
                pub fn from_builder() -> $builder_type {
                    <$builder_type as crate::PrivDefault>::priv_default()
                }
            }

            impl Resource for $type_name {}
        };
    }

    macro_rules! builder_implement_priv_default {
        ($builder_type:path) => {
            impl crate::PrivDefault for $builder_type {
                fn priv_default() -> Self {
                    $builder_type(Default::default())
                }
            }
        };
    }

    macro_rules! builder_extra_info_methods {
        ($builder_type:path) => {
            impl $builder_type {
                pub fn extra_info(
                    mut self,
                    characteristics: Option<DWORD>,
                    version: Option<DWORD>,
                ) -> Self {
                    use crate::ExtraInfo;
                    let fallback_items = (self.0).0.access_fallback_mut();
                    fallback_items.extra_info = Some(ExtraInfo {
                        characteristics,
                        version,
                    });
                    self
                }

                pub fn lang_specific_extra_info(
                    mut self,
                    lang: crate::Lang,
                    characteristics: Option<DWORD>,
                    version: Option<DWORD>,
                ) -> Self {
                    use crate::ExtraInfo;
                    let lang_items = (self.0).0.access_lang_specific_mut(lang);
                    lang_items.extra_info = Some(ExtraInfo {
                        characteristics,
                        version,
                    });
                    self
                }
            }
        };
    }

    macro_rules! builder_extra_info_methods2 {
        ($builder_type:path) => {
            impl $builder_type {
                pub fn extra_info(
                    mut self,
                    characteristics: Option<DWORD>,
                    version: Option<DWORD>,
                ) -> Self {
                    use crate::ExtraInfo;
                    (self.0).extra_info.insert_fallback(ExtraInfo {
                        characteristics,
                        version,
                    });
                    self
                }

                pub fn lang_specific_extra_info(
                    mut self,
                    lang: crate::Lang,
                    characteristics: Option<DWORD>,
                    version: Option<DWORD>,
                ) -> Self {
                    use crate::ExtraInfo;
                    (self.0).extra_info.insert_lang_specific(
                        lang,
                        ExtraInfo {
                            characteristics,
                            version,
                        },
                    );
                    self
                }
            }
        };
    }

    macro_rules! builder_build_method {
        ($builder_type:path, $type_name:path) => {
            impl $builder_type {
                pub fn build(self) -> $type_name {
                    use std::rc::Rc;
                    $type_name(Rc::new(self.0))
                }
            }
        };
    }

    macro_rules! bitflags_bitor_method {
        ($bitflags_type:path) => {
            impl std::ops::BitOr for $bitflags_type {
                type Output = $bitflags_type;

                fn bitor(self, rhs: $bitflags_type) -> $bitflags_type {
                    $bitflags_type(self.0 | rhs.0)
                }
            }
        };
    }

    macro_rules! define_builder_or_path_generated_resource {
        ($type_name:ident, $data_type:path, $builder_type:path) => {
            #[derive(Clone)]
            pub struct $type_name(pub(crate) Rc<$data_type>);

            impl $type_name {
                pub fn from_builder() -> $builder_type {
                    <$builder_type as crate::PrivDefault>::priv_default()
                }

                pub fn from_file(path: impl AsRef<Path>) -> Self {
                    create_path_only_resource_from_file(path, $type_name)
                }
            }

            impl Resource for $type_name {}
        };
    }

    define_path_only_resource!(Bitmap);
    define_path_only_resource!(Cursor);
    define_path_only_resource!(Font);
    define_path_only_resource!(HTML);
    define_path_only_resource!(Icon);
    define_path_only_resource!(MessageTable);

    define_builder_generated_resource!(
        StringTable,
        crate::string_table::StringTableData,
        crate::string_table::StringTableBuilder
    );

    define_builder_generated_resource!(
        Accelerators,
        crate::accelerators::AcceleratorsData,
        crate::accelerators::AcceleratorsBuilder
    );

    define_builder_generated_resource!(Menu, crate::menu::MenuData, crate::menu::MenuBuilder);

    define_builder_generated_resource!(
        Dialog,
        crate::dialog::DialogData,
        crate::dialog::DialogBuilder
    );

    define_builder_generated_resource!(
        VersionInfo,
        crate::version_info::VersionInfoData,
        crate::version_info::VersionInfoBuilder
    );

    define_builder_generated_resource!(
        RcInline,
        crate::rc_inline::RcInlineData,
        crate::rc_inline::RcInlineBuilder
    );

    define_builder_or_path_generated_resource!(
        UserDefined,
        crate::user_defined::UserDefinedData,
        crate::user_defined::UserDefinedBuilder
    );

    // we won't support:
    // obsolete items: plugplay vxd
    // special items: textinclude typelib
}

struct OptionLangSpecific<T>(BTreeMap<Option<Lang>, T>);

impl<T> OptionLangSpecific<T> {
    fn access_lang_specific_mut(&mut self, lang: Lang) -> &mut T
    where
        T: Default,
    {
        self.0.entry(Some(lang)).or_default()
    }

    fn access_fallback_mut(&mut self) -> &mut T
    where
        T: Default,
    {
        self.0.entry(None).or_default()
    }

    fn insert_lang_specific(&mut self, lang: Lang, v: T) {
        self.0.insert(Some(lang), v);
    }

    fn insert_fallback(&mut self, v: T) {
        self.0.insert(None, v);
    }
}

impl<T> Default for OptionLangSpecific<T> {
    fn default() -> Self {
        OptionLangSpecific(BTreeMap::default())
    }
}

pub struct ExtraInfo {
    pub characteristics: Option<DWORD>,
    pub version: Option<DWORD>,
}

trait PrivDefault {
    fn priv_default() -> Self;
}

pub mod string_table {
    use crate::{ExtraInfo, Id, Lang, OptionLangSpecific};
    use winapi::shared::minwindef::DWORD;

    #[derive(Default)]
    struct StringTableItems {
        extra_info: Option<ExtraInfo>,
        strings: Vec<(Id, String)>,
    }

    #[derive(Default)]
    pub(crate) struct StringTableData(OptionLangSpecific<StringTableItems>);

    pub struct StringTableBuilder(StringTableData);

    builder_implement_priv_default!(StringTableBuilder);
    builder_extra_info_methods!(StringTableBuilder);
    builder_build_method!(StringTableBuilder, crate::resource::StringTable);

    impl StringTableBuilder {
        pub fn string(mut self, id: impl Into<Id>, string: impl AsRef<str>) -> Self {
            let id = id.into();
            let string = string.as_ref().to_owned();
            let fallback_items = (self.0).0.access_fallback_mut();
            fallback_items.strings.push((id, string));
            self
        }

        pub fn lang_specific_string(
            mut self,
            lang: Lang,
            id: impl Into<Id>,
            string: impl AsRef<str>,
        ) -> Self {
            let id = id.into();
            let string = string.as_ref().to_owned();
            let lang_items = (self.0).0.access_lang_specific_mut(lang);
            lang_items.strings.push((id, string));
            self
        }
    }
}

pub mod accelerators {
    use crate::{ExtraInfo, Id, Lang, OptionLangSpecific};
    use std::rc::Rc;
    use winapi::ctypes::c_int;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::winuser;

    #[derive(Clone, Copy)]
    pub struct ASCIIKey(u8);

    impl ASCIIKey {
        pub fn new_checked(v: u8) -> Option<ASCIIKey> {
            match v {
                32u8..=126u8 => Some(ASCIIKey(v)),
                _ => None,
            }
        }
    }

    #[derive(Clone, Copy)]
    enum VirtKeyData {
        AlphaNumeric(u8),
        Vk(c_int),
    }

    #[derive(Clone, Copy)]
    pub struct VirtKey(VirtKeyData);

    impl VirtKey {
        pub fn alphanumeric(v: u8) -> Option<VirtKey> {
            match v {
                97u8..=122u8 => None,
                32u8..=126u8 => Some(VirtKey(VirtKeyData::AlphaNumeric(v))),
                _ => None,
            }
        }
        pub const LBUTTON: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_LBUTTON));
        pub const RBUTTON: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_RBUTTON));
        pub const CANCEL: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_CANCEL));
        pub const MBUTTON: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_MBUTTON));
        pub const XBUTTON1: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_XBUTTON1));
        pub const XBUTTON2: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_XBUTTON2));
        pub const BACK: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_BACK));
        pub const TAB: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_TAB));
        pub const CLEAR: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_CLEAR));
        pub const RETURN: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_RETURN));
        pub const SHIFT: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_SHIFT));
        pub const CONTROL: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_CONTROL));
        pub const MENU: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_MENU));
        pub const PAUSE: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_PAUSE));
        pub const CAPITAL: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_CAPITAL));
        pub const KANA: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_KANA));
        pub const HANGEUL: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_HANGEUL));
        pub const HANGUL: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_HANGUL));
        pub const JUNJA: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_JUNJA));
        pub const FINAL: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_FINAL));
        pub const HANJA: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_HANJA));
        pub const KANJI: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_KANJI));
        pub const ESCAPE: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_ESCAPE));
        pub const CONVERT: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_CONVERT));
        pub const NONCONVERT: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_NONCONVERT));
        pub const ACCEPT: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_ACCEPT));
        pub const MODECHANGE: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_MODECHANGE));
        pub const SPACE: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_SPACE));
        pub const PRIOR: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_PRIOR));
        pub const NEXT: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_NEXT));
        pub const END: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_END));
        pub const HOME: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_HOME));
        pub const LEFT: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_LEFT));
        pub const UP: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_UP));
        pub const RIGHT: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_RIGHT));
        pub const DOWN: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_DOWN));
        pub const SELECT: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_SELECT));
        pub const PRINT: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_PRINT));
        pub const EXECUTE: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_EXECUTE));
        pub const SNAPSHOT: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_SNAPSHOT));
        pub const INSERT: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_INSERT));
        pub const DELETE: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_DELETE));
        pub const HELP: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_HELP));
        pub const LWIN: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_LWIN));
        pub const RWIN: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_RWIN));
        pub const APPS: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_APPS));
        pub const SLEEP: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_SLEEP));
        pub const NUMPAD0: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_NUMPAD0));
        pub const NUMPAD1: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_NUMPAD1));
        pub const NUMPAD2: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_NUMPAD2));
        pub const NUMPAD3: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_NUMPAD3));
        pub const NUMPAD4: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_NUMPAD4));
        pub const NUMPAD5: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_NUMPAD5));
        pub const NUMPAD6: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_NUMPAD6));
        pub const NUMPAD7: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_NUMPAD7));
        pub const NUMPAD8: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_NUMPAD8));
        pub const NUMPAD9: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_NUMPAD9));
        pub const MULTIPLY: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_MULTIPLY));
        pub const ADD: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_ADD));
        pub const SEPARATOR: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_SEPARATOR));
        pub const SUBTRACT: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_SUBTRACT));
        pub const DECIMAL: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_DECIMAL));
        pub const DIVIDE: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_DIVIDE));
        pub const F1: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F1));
        pub const F2: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F2));
        pub const F3: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F3));
        pub const F4: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F4));
        pub const F5: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F5));
        pub const F6: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F6));
        pub const F7: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F7));
        pub const F8: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F8));
        pub const F9: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F9));
        pub const F10: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F10));
        pub const F11: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F11));
        pub const F12: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F12));
        pub const F13: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F13));
        pub const F14: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F14));
        pub const F15: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F15));
        pub const F16: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F16));
        pub const F17: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F17));
        pub const F18: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F18));
        pub const F19: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F19));
        pub const F20: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F20));
        pub const F21: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F21));
        pub const F22: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F22));
        pub const F23: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F23));
        pub const F24: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_F24));
        pub const NUMLOCK: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_NUMLOCK));
        pub const SCROLL: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_SCROLL));
        pub const OEM_NEC_EQUAL: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_NEC_EQUAL));
        pub const OEM_FJ_JISHO: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_FJ_JISHO));
        pub const OEM_FJ_MASSHOU: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_FJ_MASSHOU));
        pub const OEM_FJ_TOUROKU: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_FJ_TOUROKU));
        pub const OEM_FJ_LOYA: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_FJ_LOYA));
        pub const OEM_FJ_ROYA: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_FJ_ROYA));
        pub const LSHIFT: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_LSHIFT));
        pub const RSHIFT: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_RSHIFT));
        pub const LCONTROL: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_LCONTROL));
        pub const RCONTROL: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_RCONTROL));
        pub const LMENU: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_LMENU));
        pub const RMENU: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_RMENU));
        pub const BROWSER_BACK: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_BROWSER_BACK));
        pub const BROWSER_FORWARD: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_BROWSER_FORWARD));
        pub const BROWSER_REFRESH: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_BROWSER_REFRESH));
        pub const BROWSER_STOP: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_BROWSER_STOP));
        pub const BROWSER_SEARCH: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_BROWSER_SEARCH));
        pub const BROWSER_FAVORITES: VirtKey =
            VirtKey(VirtKeyData::Vk(winuser::VK_BROWSER_FAVORITES));
        pub const BROWSER_HOME: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_BROWSER_HOME));
        pub const VOLUME_MUTE: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_VOLUME_MUTE));
        pub const VOLUME_DOWN: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_VOLUME_DOWN));
        pub const VOLUME_UP: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_VOLUME_UP));
        pub const MEDIA_NEXT_TRACK: VirtKey =
            VirtKey(VirtKeyData::Vk(winuser::VK_MEDIA_NEXT_TRACK));
        pub const MEDIA_PREV_TRACK: VirtKey =
            VirtKey(VirtKeyData::Vk(winuser::VK_MEDIA_PREV_TRACK));
        pub const MEDIA_STOP: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_MEDIA_STOP));
        pub const MEDIA_PLAY_PAUSE: VirtKey =
            VirtKey(VirtKeyData::Vk(winuser::VK_MEDIA_PLAY_PAUSE));
        pub const LAUNCH_MAIL: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_LAUNCH_MAIL));
        pub const LAUNCH_MEDIA_SELECT: VirtKey =
            VirtKey(VirtKeyData::Vk(winuser::VK_LAUNCH_MEDIA_SELECT));
        pub const LAUNCH_APP1: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_LAUNCH_APP1));
        pub const LAUNCH_APP2: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_LAUNCH_APP2));
        pub const OEM_1: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_1));
        pub const OEM_PLUS: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_PLUS));
        pub const OEM_COMMA: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_COMMA));
        pub const OEM_MINUS: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_MINUS));
        pub const OEM_PERIOD: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_PERIOD));
        pub const OEM_2: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_2));
        pub const OEM_3: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_3));
        pub const OEM_4: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_4));
        pub const OEM_5: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_5));
        pub const OEM_6: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_6));
        pub const OEM_7: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_7));
        pub const OEM_8: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_8));
        pub const OEM_AX: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_AX));
        pub const OEM_102: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_102));
        pub const ICO_HELP: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_ICO_HELP));
        pub const ICO_00: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_ICO_00));
        pub const PROCESSKEY: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_PROCESSKEY));
        pub const ICO_CLEAR: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_ICO_CLEAR));
        pub const PACKET: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_PACKET));
        pub const OEM_RESET: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_RESET));
        pub const OEM_JUMP: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_JUMP));
        pub const OEM_PA1: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_PA1));
        pub const OEM_PA2: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_PA2));
        pub const OEM_PA3: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_PA3));
        pub const OEM_WSCTRL: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_WSCTRL));
        pub const OEM_CUSEL: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_CUSEL));
        pub const OEM_ATTN: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_ATTN));
        pub const OEM_FINISH: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_FINISH));
        pub const OEM_COPY: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_COPY));
        pub const OEM_AUTO: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_AUTO));
        pub const OEM_ENLW: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_ENLW));
        pub const OEM_BACKTAB: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_BACKTAB));
        pub const ATTN: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_ATTN));
        pub const CRSEL: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_CRSEL));
        pub const EXSEL: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_EXSEL));
        pub const EREOF: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_EREOF));
        pub const PLAY: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_PLAY));
        pub const ZOOM: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_ZOOM));
        pub const NONAME: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_NONAME));
        pub const PA1: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_PA1));
        pub const OEM_CLEAR: VirtKey = VirtKey(VirtKeyData::Vk(winuser::VK_OEM_CLEAR));
    }

    #[derive(Clone, Copy)]
    pub enum Modifier {
        None,
        Ctrl,
        Alt,
        Shift,
        CtrlAlt,
        CtrlShift,
        AltShift,
        CtrlAltShift,
    }

    #[derive(Clone, Copy)]
    pub enum ASCIIModifier {
        None,
        Ctrl,
    }

    #[derive(Clone, Copy)]
    enum Key {
        ASCII {
            ascii_key: ASCIIKey,
            modifier: ASCIIModifier,
        },
        VirtKey {
            virt_key: VirtKey,
            modifier: Modifier,
        },
    }

    #[derive(Clone, Copy)]
    pub struct Event {
        key: Key,
        noinvert: bool,
    }

    impl Event {
        pub const fn virt_key_event(virt_key: VirtKey, modifier: Modifier) -> Self {
            Event {
                key: Key::VirtKey { virt_key, modifier },
                noinvert: false,
            }
        }

        pub const fn ascii_key_event(ascii_key: ASCIIKey, modifier: ASCIIModifier) -> Self {
            Event {
                key: Key::ASCII {
                    ascii_key,
                    modifier,
                },
                noinvert: false,
            }
        }

        #[deprecated]
        pub const fn noinvert(mut self) -> Self {
            self.noinvert = true;
            self
        }
    }

    #[derive(Default)]
    struct AcceleratorsItems {
        extra_info: Option<ExtraInfo>,
        events: Vec<(Id, Event)>,
    }

    #[derive(Default)]
    pub(crate) struct AcceleratorsData(OptionLangSpecific<AcceleratorsItems>);

    pub struct AcceleratorsBuilder(AcceleratorsData);

    builder_implement_priv_default!(AcceleratorsBuilder);
    builder_extra_info_methods!(AcceleratorsBuilder);
    builder_build_method!(AcceleratorsBuilder, crate::resource::Accelerators);

    impl AcceleratorsBuilder {
        pub fn string(mut self, id: impl Into<Id>, event: Event) -> Self {
            let id = id.into();
            let common_items = (self.0).0.access_fallback_mut();
            common_items.events.push((id, event));
            self
        }

        pub fn lang_specific_string(mut self, lang: Lang, id: impl Into<Id>, event: Event) -> Self {
            let id = id.into();
            let lang_items = (self.0).0.access_lang_specific_mut(lang);
            lang_items.events.push((id, event));
            self
        }
    }
}

pub mod menu {
    use crate::{CowStr, Id, OptionLangSpecific};
    use std::rc::Rc;
    use winapi::ctypes::c_int;
    use winapi::shared::minwindef::UINT;
    use winapi::um::winuser;

    #[derive(Clone, Copy, Default)]
    pub struct MenuType(UINT);

    impl MenuType {
        // pub const STRING: MenuType = MenuType(winuser::MFT_STRING); //zero, not needed.
        pub const BITMAP: MenuType = MenuType(winuser::MFT_BITMAP);
        pub const MENUBAR_BREAK: MenuType = MenuType(winuser::MFT_MENUBARBREAK);
        pub const MENU_BREAK: MenuType = MenuType(winuser::MFT_MENUBREAK);
        pub const OWNER_DRAW: MenuType = MenuType(winuser::MFT_OWNERDRAW);
        pub const RADIO_CHECK: MenuType = MenuType(winuser::MFT_RADIOCHECK);
        pub const SEPARATOR: MenuType = MenuType(winuser::MFT_SEPARATOR);
        pub const RIGHT_ORDER: MenuType = MenuType(winuser::MFT_RIGHTORDER);
        pub const RIGHT_JUSTIFY: MenuType = MenuType(winuser::MFT_RIGHTJUSTIFY);
    }

    bitflags_bitor_method!(MenuType);

    #[derive(Clone, Copy, Default)]
    pub struct MenuState(UINT);

    impl MenuState {
        // pub const GRAYED: MenuState = MenuState(winuser::MFS_GRAYED); // alias of DISABLED
        pub const DISABLED: MenuState = MenuState(winuser::MFS_DISABLED);
        pub const CHECKED: MenuState = MenuState(winuser::MFS_CHECKED);
        pub const HILITE: MenuState = MenuState(winuser::MFS_HILITE);
        // pub const ENABLED: MenuState = MenuState(winuser::MFS_ENABLED); // zero, not needed
        // pub const UNCHECKED: MenuState = MenuState(winuser::MFS_UNCHECKED); // zero, not needed
        // pub const UNHILITE: MenuState = MenuState(winuser::MFS_UNHILITE); // zero, not needed
        pub const DEFAULT: MenuState = MenuState(winuser::MFS_DEFAULT);
    }

    bitflags_bitor_method!(MenuState);

    #[derive(Default)]
    struct PopupData {
        help_id: Option<c_int>,
        items: Vec<MenuItem>,
    }

    struct MenuItem {
        id: Id,
        text: OptionLangSpecific<CowStr>,
        ty: MenuType,
        state: MenuState,
        popup: Option<PopupData>,
    }

    #[derive(Default)]
    pub(crate) struct MenuData(Vec<MenuItem>);

    pub struct MenuBuilder(MenuData);

    builder_implement_priv_default!(MenuBuilder);
    builder_build_method!(MenuBuilder, crate::resource::Menu);
}

pub mod dialog {
    use crate::{CowStr, ExtraInfo, Id, IdOrName, OptionLangSpecific};
    use winapi::ctypes::c_int;
    use winapi::ctypes::c_long;
    use winapi::shared::minwindef::{BOOL, BYTE, DWORD};
    use winapi::shared::minwindef::{FALSE, TRUE};
    use winapi::um::wingdi;

    pub struct Rect {
        x: c_int,
        y: c_int,
        width: c_int,
        height: c_int,
    }

    pub struct Font {
        pointsize: c_int,
        typeface: CowStr,
        weight: FontWeight,
        italic: FontItalic,
        charset: FontCharset,
    }

    #[derive(Default)]
    pub struct FontWeight(c_long);

    impl FontWeight {
        // pub const DONTCARE: FontWeight = FontWeight(wingdi::FW_DONTCARE);  //zero, use FontWeight::default()
        pub const THIN: FontWeight = FontWeight(wingdi::FW_THIN);
        pub const EXTRALIGHT: FontWeight = FontWeight(wingdi::FW_EXTRALIGHT);
        pub const LIGHT: FontWeight = FontWeight(wingdi::FW_LIGHT);
        pub const NORMAL: FontWeight = FontWeight(wingdi::FW_NORMAL);
        pub const MEDIUM: FontWeight = FontWeight(wingdi::FW_MEDIUM);
        pub const SEMIBOLD: FontWeight = FontWeight(wingdi::FW_SEMIBOLD);
        pub const BOLD: FontWeight = FontWeight(wingdi::FW_BOLD);
        pub const EXTRABOLD: FontWeight = FontWeight(wingdi::FW_EXTRABOLD);
        pub const HEAVY: FontWeight = FontWeight(wingdi::FW_HEAVY);
        // pub const ULTRALIGHT: FontWeight = FontWeight(wingdi::FW_ULTRALIGHT); // alias of EXTRALIGHT
        // pub const REGULAR: FontWeight = FontWeight(wingdi::FW_REGULAR); // alias of NORMAL
        // pub const DEMIBOLD: FontWeight = FontWeight(wingdi::FW_DEMIBOLD); // alias of SEMIBOLD
        // pub const ULTRABOLD: FontWeight = FontWeight(wingdi::FW_ULTRABOLD); // alias of EXTRABOLD
        // pub const BLACK: FontWeight = FontWeight(wingdi::FW_BLACK); // alias of HEAVY
    }

    #[derive(Default)]
    pub struct FontItalic(BOOL);

    impl FontItalic {
        // const NORMAL: FontItalic = FontItalic(FALSE); // zero, use FontItalic::default()
        const ITALIC: FontItalic = FontItalic(TRUE);
    }

    pub struct FontCharset(BYTE);

    impl Default for FontCharset {
        fn default() -> Self {
            FontCharset(wingdi::DEFAULT_CHARSET as _)
        }
    }

    impl FontCharset {
        //pub const DEFAULT: FontCharset = FontCharset(wingdi::DEFAULT_CHARSET as _); // default, use FontCharset::default()
        pub const ANSI: FontCharset = FontCharset(wingdi::ANSI_CHARSET as _);
        pub const OEM: FontCharset = FontCharset(wingdi::OEM_CHARSET as _);
        pub const MAC: FontCharset = FontCharset(wingdi::MAC_CHARSET as _);
        pub const SYMBOL: FontCharset = FontCharset(wingdi::SYMBOL_CHARSET as _);
        pub const SHIFTJIS: FontCharset = FontCharset(wingdi::SHIFTJIS_CHARSET as _);
        pub const HANGEUL: FontCharset = FontCharset(wingdi::HANGEUL_CHARSET as _);
        pub const HANGUL: FontCharset = FontCharset(wingdi::HANGUL_CHARSET as _);
        pub const GB2312: FontCharset = FontCharset(wingdi::GB2312_CHARSET as _);
        pub const CHINESEBIG5: FontCharset = FontCharset(wingdi::CHINESEBIG5_CHARSET as _);
        pub const JOHAB: FontCharset = FontCharset(wingdi::JOHAB_CHARSET as _);
        pub const HEBREW: FontCharset = FontCharset(wingdi::HEBREW_CHARSET as _);
        pub const ARABIC: FontCharset = FontCharset(wingdi::ARABIC_CHARSET as _);
        pub const GREEK: FontCharset = FontCharset(wingdi::GREEK_CHARSET as _);
        pub const TURKISH: FontCharset = FontCharset(wingdi::TURKISH_CHARSET as _);
        pub const VIETNAMESE: FontCharset = FontCharset(wingdi::VIETNAMESE_CHARSET as _);
        pub const THAI: FontCharset = FontCharset(wingdi::THAI_CHARSET as _);
        pub const EASTEUROPE: FontCharset = FontCharset(wingdi::EASTEUROPE_CHARSET as _);
        pub const RUSSIAN: FontCharset = FontCharset(wingdi::RUSSIAN_CHARSET as _);
        pub const BALTIC: FontCharset = FontCharset(wingdi::BALTIC_CHARSET as _);
    }

    struct DialogControlStyle(Option<DWORD>, Option<DWORD>);

    pub struct DialogControl {
        template: ControlTemplate,
        id: Id,
        text: OptionLangSpecific<CowStr>,
        rect: Rect,
        class: Option<CowStr>,
        style: Option<DialogControlStyle>,
    }

    pub struct ControlTemplate {
        name: &'static str,
        use_text: bool,
        use_size: bool,
        use_keyword: Option<&'static str>,
    }

    impl ControlTemplate {
        pub const CONTROL: ControlTemplate = ControlTemplate {
            name: "CONTROL",
            use_text: true,
            use_size: true,
            use_keyword: None,
        };
        pub const AUTO3STATE: ControlTemplate = ControlTemplate {
            name: "AUTO3STATE",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const AUTOCHECKBOX: ControlTemplate = ControlTemplate {
            name: "AUTOCHECKBOX",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const AUTORADIOBUTTON: ControlTemplate = ControlTemplate {
            name: "AUTORADIOBUTTON",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const CHECKBOX: ControlTemplate = ControlTemplate {
            name: "CHECKBOX",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const COMBOBOX: ControlTemplate = ControlTemplate {
            name: "COMBOBOX",
            use_text: false,
            use_size: true,
            use_keyword: Some("COMBOBOX"),
        };
        pub const CTEXT: ControlTemplate = ControlTemplate {
            name: "CTEXT",
            use_text: true,
            use_size: true,
            use_keyword: Some("STATIC"),
        };
        pub const DEFPUSHBUTTON: ControlTemplate = ControlTemplate {
            name: "DEFPUSHBUTTON",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const EDITTEXT: ControlTemplate = ControlTemplate {
            name: "EDITTEXT",
            use_text: true,
            use_size: true,
            use_keyword: Some("EDIT"),
        };
        pub const GROUPBOX: ControlTemplate = ControlTemplate {
            name: "GROUPBOX",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const ICON: ControlTemplate = ControlTemplate {
            name: "ICON",
            use_text: true,
            use_size: false,
            use_keyword: Some("STATIC"),
        };
        pub const LISTBOX: ControlTemplate = ControlTemplate {
            name: "LISTBOX",
            use_text: false,
            use_size: true,
            use_keyword: Some("LISTBOX"),
        };
        pub const LTEXT: ControlTemplate = ControlTemplate {
            name: "LTEXT",
            use_text: true,
            use_size: true,
            use_keyword: Some("STATIC"),
        };
        pub const PUSHBOX: ControlTemplate = ControlTemplate {
            name: "PUSHBOX",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const PUSHBUTTON: ControlTemplate = ControlTemplate {
            name: "PUSHBUTTON",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const RADIOBUTTON: ControlTemplate = ControlTemplate {
            name: "RADIOBUTTON",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
        pub const RTEXT: ControlTemplate = ControlTemplate {
            name: "RTEXT",
            use_text: true,
            use_size: true,
            use_keyword: Some("STATIC"),
        };
        pub const SCROLLBAR: ControlTemplate = ControlTemplate {
            name: "SCROLLBAR",
            use_text: true,
            use_size: true,
            use_keyword: Some("SCROLLBAR"),
        };
        pub const STATE3: ControlTemplate = ControlTemplate {
            name: "STATE3",
            use_text: true,
            use_size: true,
            use_keyword: Some("BUTTON"),
        };
    }

    pub struct DialogStyle(DWORD, DWORD);

    #[derive(Default)]
    pub(crate) struct DialogData {
        rect: OptionLangSpecific<Rect>,
        help_id: OptionLangSpecific<c_int>,
        extra_info: OptionLangSpecific<ExtraInfo>,
        caption: OptionLangSpecific<CowStr>,
        class: Option<IdOrName>,
        style: Option<DialogStyle>,
        font: Option<Font>,
        menu: Option<IdOrName>,
        controls: Vec<DialogControl>,
    }

    pub struct DialogBuilder(DialogData);

    builder_implement_priv_default!(DialogBuilder);
    builder_build_method!(DialogBuilder, crate::resource::Dialog);

}

pub mod version_info {
    use crate::CowStr;
    use crate::OptionLangSpecific;
    use winapi::shared::minwindef::{DWORD, WORD};

    pub struct Version([WORD; 4]);
    pub struct FileFlags(DWORD);
    pub struct FileOS(DWORD);
    pub struct FileType(DWORD);

    #[derive(Default)]
    pub(crate) struct VersionInfoData {
        fixed_file_version: Option<Version>,
        fixed_product_version: Option<Version>,
        fixed_file_flags: Option<FileFlags>,
        fixed_file_os: Option<FileOS>,
        fixed_file_type: Option<FileType>,
        product_name: OptionLangSpecific<CowStr>,
        product_version: OptionLangSpecific<CowStr>,
        file_description: OptionLangSpecific<CowStr>,
        file_version: OptionLangSpecific<CowStr>,
        internal_name: OptionLangSpecific<CowStr>,
        original_filename: OptionLangSpecific<CowStr>,
        company_name: OptionLangSpecific<CowStr>,
        legal_copyright: Option<OptionLangSpecific<CowStr>>,
        legal_trademarks: Option<OptionLangSpecific<CowStr>>,
        private_build: Option<OptionLangSpecific<CowStr>>,
        special_build: Option<OptionLangSpecific<CowStr>>,
        comments: Option<OptionLangSpecific<CowStr>>,
    }

    //we only support Unicode as charset here.

    pub struct VersionInfoBuilder(VersionInfoData);

    builder_implement_priv_default!(VersionInfoBuilder);
    builder_build_method!(VersionInfoBuilder, crate::resource::VersionInfo);
}

pub mod rc_inline {
    use crate::{ExtraInfo, OptionLangSpecific};
    use winapi::shared::minwindef::{DWORD, WORD};

    enum RcInlineItem {
        U16(WORD),
        U32(DWORD),
        Str(Vec<u8>),
        WStr(Vec<u16>),
    }

    #[derive(Default)]
    pub(crate) struct RcInlineData {
        extra_info: OptionLangSpecific<ExtraInfo>,
        items: OptionLangSpecific<Vec<RcInlineItem>>,
    }

    pub struct RcInlineBuilder(RcInlineData);
    builder_implement_priv_default!(RcInlineBuilder);
    builder_extra_info_methods2!(RcInlineBuilder);
    builder_build_method!(RcInlineBuilder, crate::resource::RcInline);
}

pub mod user_defined {
    use crate::rc_inline::RcInlineData;
    use crate::CowPath;

    pub(crate) enum UserDefinedData {
        RcInline(RcInlineData),
        External(CowPath),
    }

    impl Default for UserDefinedData {
        fn default() -> Self {
            UserDefinedData::RcInline(Default::default())
        }
    }

    impl From<CowPath> for UserDefinedData {
        fn from(path: CowPath) -> Self {
            UserDefinedData::External(path)
        }
    }

    pub struct UserDefinedBuilder(UserDefinedData);
    builder_implement_priv_default!(UserDefinedBuilder);
    builder_build_method!(UserDefinedBuilder, crate::resource::UserDefined);
}
