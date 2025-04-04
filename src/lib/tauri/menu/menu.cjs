'use strict';

var submenu = require('./submenu.cjs');
var dpi = require('../dpi.cjs');
var core = require('../core.cjs');
var base = require('./base.cjs');

// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
/** A type that is either a menu bar on the window
 * on Windows and Linux or as a global menu in the menubar on macOS.
 *
 * #### Platform-specific:
 *
 * - **macOS**: if using {@linkcode Menu} for the global menubar, it can only contain {@linkcode Submenu}s.
 */
class Menu extends base.MenuItemBase {
    /** @ignore */
    constructor(rid, id) {
        super(rid, id, 'Menu');
    }
    /** Create a new menu. */
    static async new(opts) {
        return base.newMenu('Menu', opts).then(([rid, id]) => new Menu(rid, id));
    }
    /** Create a default menu. */
    static async default() {
        return core.invoke('plugin:menu|create_default').then(([rid, id]) => new Menu(rid, id));
    }
    /**
     * Add a menu item to the end of this menu.
     *
     * #### Platform-specific:
     *
     * - **macOS:** Only {@linkcode Submenu}s can be added to a {@linkcode Menu}.
     */
    async append(items) {
        return core.invoke('plugin:menu|append', {
            rid: this.rid,
            kind: this.kind,
            items: (Array.isArray(items) ? items : [items]).map((i) => 'rid' in i ? [i.rid, i.kind] : i)
        });
    }
    /**
     * Add a menu item to the beginning of this menu.
     *
     * #### Platform-specific:
     *
     * - **macOS:** Only {@linkcode Submenu}s can be added to a {@linkcode Menu}.
     */
    async prepend(items) {
        return core.invoke('plugin:menu|prepend', {
            rid: this.rid,
            kind: this.kind,
            items: (Array.isArray(items) ? items : [items]).map((i) => 'rid' in i ? [i.rid, i.kind] : i)
        });
    }
    /**
     * Add a menu item to the specified position in this menu.
     *
     * #### Platform-specific:
     *
     * - **macOS:** Only {@linkcode Submenu}s can be added to a {@linkcode Menu}.
     */
    async insert(items, position) {
        return core.invoke('plugin:menu|insert', {
            rid: this.rid,
            kind: this.kind,
            items: (Array.isArray(items) ? items : [items]).map((i) => 'rid' in i ? [i.rid, i.kind] : i),
            position
        });
    }
    /** Remove a menu item from this menu. */
    async remove(item) {
        return core.invoke('plugin:menu|remove', {
            rid: this.rid,
            kind: this.kind,
            item: [item.rid, item.kind]
        });
    }
    /** Remove a menu item from this menu at the specified position. */
    async removeAt(position) {
        return core.invoke('plugin:menu|remove_at', {
            rid: this.rid,
            kind: this.kind,
            position
        }).then(submenu.itemFromKind);
    }
    /** Returns a list of menu items that has been added to this menu. */
    async items() {
        return core.invoke('plugin:menu|items', {
            rid: this.rid,
            kind: this.kind
        }).then((i) => i.map(submenu.itemFromKind));
    }
    /** Retrieves the menu item matching the given identifier. */
    async get(id) {
        return core.invoke('plugin:menu|get', {
            rid: this.rid,
            kind: this.kind,
            id
        }).then((r) => (r ? submenu.itemFromKind(r) : null));
    }
    /**
     * Popup this menu as a context menu on the specified window.
     *
     * If the position, is provided, it is relative to the window's top-left corner.
     */
    async popup(at, window) {
        var _a;
        return core.invoke('plugin:menu|popup', {
            rid: this.rid,
            kind: this.kind,
            window: (_a = window === null || window === void 0 ? void 0 : window.label) !== null && _a !== void 0 ? _a : null,
            at: at instanceof dpi.Position ? at : at ? new dpi.Position(at) : null
        });
    }
    /**
     * Sets the app-wide menu and returns the previous one.
     *
     * If a window was not created with an explicit menu or had one set explicitly,
     * this menu will be assigned to it.
     */
    async setAsAppMenu() {
        return core.invoke('plugin:menu|set_as_app_menu', {
            rid: this.rid
        }).then((r) => (r ? new Menu(r[0], r[1]) : null));
    }
    /**
     * Sets the window menu and returns the previous one.
     *
     * #### Platform-specific:
     *
     * - **macOS:** Unsupported. The menu on macOS is app-wide and not specific to one
     * window, if you need to set it, use {@linkcode Menu.setAsAppMenu} instead.
     */
    async setAsWindowMenu(window) {
        var _a;
        return core.invoke('plugin:menu|set_as_window_menu', {
            rid: this.rid,
            window: (_a = window === null || window === void 0 ? void 0 : window.label) !== null && _a !== void 0 ? _a : null
        }).then((r) => (r ? new Menu(r[0], r[1]) : null));
    }
}

exports.Menu = Menu;
