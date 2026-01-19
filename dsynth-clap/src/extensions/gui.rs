//! CLAP GUI extension implementation (CLAP_EXT_GUI)

use crate::{instance::PluginInstance, plugin::ClapPlugin};
use clap_sys::ext::gui::*;
use std::ffi::CStr;
use std::sync::OnceLock;

/// Get the GUI extension for a plugin type.
///
/// Plugins can opt-in by overriding `ClapPlugin::has_gui()` and the `gui_*` hooks.
pub fn get_extension<P: ClapPlugin>() -> &'static clap_plugin_gui {
    static EXT: OnceLock<clap_plugin_gui> = OnceLock::new();
    EXT.get_or_init(|| clap_plugin_gui {
        is_api_supported: Some(gui_is_api_supported::<P>),
        get_preferred_api: Some(gui_get_preferred_api::<P>),
        create: Some(gui_create::<P>),
        destroy: Some(gui_destroy::<P>),
        set_scale: Some(gui_set_scale::<P>),
        get_size: Some(gui_get_size::<P>),
        can_resize: Some(gui_can_resize::<P>),
        get_resize_hints: None,
        adjust_size: None,
        set_size: Some(gui_set_size::<P>),
        set_parent: Some(gui_set_parent::<P>),
        set_transient: None,
        suggest_title: None,
        show: Some(gui_show::<P>),
        hide: Some(gui_hide::<P>),
    })
}

unsafe extern "C" fn gui_is_api_supported<P: ClapPlugin>(
    plugin: *const clap_sys::plugin::clap_plugin,
    api: *const i8,
    is_floating: bool,
) -> bool {
    if plugin.is_null() || api.is_null() {
        return false;
    }
    if !P::has_gui() {
        return false;
    }

    let api_str = match CStr::from_ptr(api).to_str() {
        Ok(_) => CStr::from_ptr(api),
        Err(_) => return false,
    };

    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    instance.plugin.gui_is_api_supported(api_str, is_floating)
}

unsafe extern "C" fn gui_get_preferred_api<P: ClapPlugin>(
    plugin: *const clap_sys::plugin::clap_plugin,
    api: *mut *const i8,
    is_floating: *mut bool,
) -> bool {
    if plugin.is_null() || api.is_null() || is_floating.is_null() {
        return false;
    }
    if !P::has_gui() {
        return false;
    }

    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    instance.plugin.gui_get_preferred_api(api, is_floating)
}

unsafe extern "C" fn gui_create<P: ClapPlugin>(
    plugin: *const clap_sys::plugin::clap_plugin,
    api: *const i8,
    is_floating: bool,
) -> bool {
    if plugin.is_null() || api.is_null() {
        return false;
    }
    if !P::has_gui() {
        return false;
    }

    let api_str = CStr::from_ptr(api);
    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    instance.plugin.gui_create(api_str, is_floating)
}

unsafe extern "C" fn gui_destroy<P: ClapPlugin>(plugin: *const clap_sys::plugin::clap_plugin) {
    if plugin.is_null() {
        return;
    }
    if !P::has_gui() {
        return;
    }

    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    instance.plugin.gui_destroy();
}

unsafe extern "C" fn gui_set_scale<P: ClapPlugin>(
    plugin: *const clap_sys::plugin::clap_plugin,
    scale: f64,
) -> bool {
    if plugin.is_null() {
        return false;
    }
    if !P::has_gui() {
        return false;
    }

    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    instance.plugin.gui_set_scale(scale)
}

unsafe extern "C" fn gui_get_size<P: ClapPlugin>(
    plugin: *const clap_sys::plugin::clap_plugin,
    width: *mut u32,
    height: *mut u32,
) -> bool {
    if plugin.is_null() || width.is_null() || height.is_null() {
        return false;
    }
    if !P::has_gui() {
        return false;
    }

    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    instance.plugin.gui_get_size(width, height)
}

unsafe extern "C" fn gui_can_resize<P: ClapPlugin>(
    plugin: *const clap_sys::plugin::clap_plugin,
) -> bool {
    if plugin.is_null() {
        return false;
    }
    if !P::has_gui() {
        return false;
    }

    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    instance.plugin.gui_can_resize()
}

unsafe extern "C" fn gui_set_size<P: ClapPlugin>(
    plugin: *const clap_sys::plugin::clap_plugin,
    width: u32,
    height: u32,
) -> bool {
    if plugin.is_null() {
        return false;
    }
    if !P::has_gui() {
        return false;
    }

    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    instance.plugin.gui_set_size(width, height)
}

unsafe extern "C" fn gui_set_parent<P: ClapPlugin>(
    plugin: *const clap_sys::plugin::clap_plugin,
    window: *const clap_window,
) -> bool {
    if plugin.is_null() {
        return false;
    }
    if !P::has_gui() {
        return false;
    }

    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    instance.plugin.gui_set_parent(window)
}

unsafe extern "C" fn gui_show<P: ClapPlugin>(plugin: *const clap_sys::plugin::clap_plugin) -> bool {
    if plugin.is_null() {
        return false;
    }
    if !P::has_gui() {
        return false;
    }

    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    instance.plugin.gui_show()
}

unsafe extern "C" fn gui_hide<P: ClapPlugin>(plugin: *const clap_sys::plugin::clap_plugin) -> bool {
    if plugin.is_null() {
        return false;
    }
    if !P::has_gui() {
        return false;
    }

    let instance = PluginInstance::<P>::from_ptr_mut(plugin);
    instance.plugin.gui_hide()
}
