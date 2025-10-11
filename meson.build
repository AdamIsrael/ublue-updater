project(
  'ublue-updater',
  'rust',
  version: '0.1',
  default_options: ['warning_level=3', 'rust_std=2024'],

)

# i18n = import('i18n')
gnome = import('gnome')

subdir('src')

# rust_compiler = meson.get_compiler('rust')
#   my_crate_dep = rust_compiler.find_library(
#   'my_external_crate',
#   dirs: ['path/to/my_external_crate/target/debug']
# )

# executable(
#   'ublue-updater', 'src/main.rs', install: true,
#   # dependencies: [my_crate_dep]
# )

gnome.post_install(
  glib_compile_schemas: true,
  gtk_update_icon_cache: true,
  update_desktop_database: true,
)
