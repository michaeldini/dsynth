// Test CLAP plugin loading
#include <stdio.h>
#include <dlfcn.h>
#include <stdint.h>

typedef struct clap_version {
    uint32_t major;
    uint32_t minor;
    uint32_t revision;
} clap_version_t;

typedef struct clap_plugin_entry {
    clap_version_t clap_version;
    int (*init)(const char *);
    void (*deinit)(void);
    const void* (*get_factory)(const char *);
} clap_plugin_entry_t;

// CLAP entry point can be either a symbol or a function
typedef const clap_plugin_entry_t* (*clap_entry_func_t)(const char*);

int main() {
    const char *path = "/Users/michaeldini/Library/Audio/Plug-Ins/CLAP/SimpleSynth.clap/Contents/MacOS/SimpleSynth";
    
    printf("Loading: %s\n", path);
    void *handle = dlopen(path, RTLD_NOW | RTLD_LOCAL);
    
    if (!handle) {
        printf("ERROR: Failed to load library: %s\n", dlerror());
        return 1;
    }
    
    printf("✓ Library loaded\n");
    
    // Prefer the CLAP-required static symbol
    clap_plugin_entry_t *entry = dlsym(handle, "clap_entry");
    if (entry) {
        printf("✓ clap_entry static found\n");
    } else {
        // Fallback: some hosts/tooling may expose a function returning the entry
        clap_entry_func_t entry_func = dlsym(handle, "get_clap_entry");
        if (!entry_func) {
            printf("ERROR: Failed to find clap_entry or get_clap_entry: %s\n", dlerror());
            dlclose(handle);
            return 1;
        }
        printf("✓ get_clap_entry function found\n");
        printf("Calling get_clap_entry()...\n");
        entry = (clap_plugin_entry_t*)entry_func(path);
        if (!entry) {
            printf("ERROR: get_clap_entry() returned NULL\n");
            dlclose(handle);
            return 1;
        }
    }
    
    printf("  CLAP version: %u.%u.%u\n", entry->clap_version.major, entry->clap_version.minor, entry->clap_version.revision);
    printf("  init: %p\n", entry->init);
    printf("  deinit: %p\n", entry->deinit);
    printf("  get_factory: %p\n", entry->get_factory);
    
    if (!entry->init) {
        printf("ERROR: init is NULL\n");
        dlclose(handle);
        return 1;
    }
    
    printf("\nCalling init()...\n");
    int result = entry->init(path);
    printf("  init() returned: %d\n", result);
    
    if (!entry->get_factory) {
        printf("ERROR: get_factory is NULL\n");
        dlclose(handle);
        return 1;
    }
    
    printf("\nCalling get_factory(\"clap.plugin-factory\")...\n");
    const void *factory = entry->get_factory("clap.plugin-factory");
    printf("  Factory: %p\n", factory);
    
    if (entry->deinit) {
        printf("\nCalling deinit()...\n");
        entry->deinit();
    }
    
    dlclose(handle);
    printf("\n✓ All tests passed!\n");
    return 0;
}
