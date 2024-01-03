use std::fs::File;
use std::io::Read;
use egui::Ui;
use mlua::{Function, Lua, Table, UserData, UserDataMethods};
use crate::app::generators::control::{Control};
use crate::app::generators::ui_elements::{CheckboxInput, ComboBoxInput, Slider, TextInput};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ControlHandler {
    pub(crate) generators: Vec<LuaGenerator>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct LuaGenerator {
    #[serde(skip)]
    pub lua: Lua,
    pub script: String,
    pub script_data: String,
    pub controls: Vec<Control>,
    #[serde(skip)]
    pub loaded: bool,
}

impl ControlHandler {
    pub fn render(&mut self, ui: &mut Ui) {
        self.tick_lua();

        self.add_controls();

        self.render_controls(ui);

    }

    pub fn add_controls(&mut self)
    {
        let mut i = 0;
        for generator in &mut self.generators {
            let mut new_controls = vec![];

            let get_controls: Function<'_> = generator.lua.globals().get("get_controls").unwrap();
            let controls_changed: Function<'_> = generator.lua.globals().get("controls_changed").unwrap();

            let changed = controls_changed.call::<_, bool>(()).unwrap();

            if changed == false {
                // ensure unique IDs for each control
                i += 100;
                continue;
            }

            generator.controls.clear();

            let table = get_controls.call::<_, Table<'_>>(()).unwrap();

            table.for_each(|k: String,
                            v: Table<'_>| {
                i += 1;
                match v.raw_get::<i32, String>(1) {
                    Ok(s) => {
                        match s.as_str() {
                            "Slider" => {
                                let control = Control::SliderType(Slider {
                                    name: v.raw_get::<i32, String>(2).unwrap(),
                                    label: v.raw_get::<i32, String>(3).unwrap(),
                                    min: v.raw_get::<i32, f32>(4).unwrap(),
                                    max: v.raw_get::<i32, f32>(5).unwrap(),
                                    value: v.raw_get::<i32, f32>(6).unwrap(),
                                    step_by: v.raw_get::<i32, f64>(7).unwrap(),
                                    deicimals: v.raw_get::<i32, usize>(8).unwrap(),
                                    keybinding: None,
                                });
                                generator.lua.globals().set(k, control.clone()).unwrap();
                                new_controls.push(control);
                            }
                            "TextInput" => {
                                let control = Control::TextInputType(TextInput {
                                    name: v.raw_get::<i32, String>(2).unwrap(),
                                    label: v.raw_get::<i32, String>(3).unwrap(),
                                    value: v.raw_get::<i32, String>(4).unwrap(),
                                });
                                generator.lua.globals().set(k, control.clone()).unwrap();
                                new_controls.push(control);
                            }
                            "ComboBox" => {
                                let mut entries = vec![];
                                for i in 5..50 {
                                    match v.raw_get::<i32, String>(i) {
                                        Ok(s) => { entries.push(s); }
                                        Err(_) => { break; }
                                    }
                                }

                                let control = Control::ComboBoxType(ComboBoxInput {
                                    name: v.raw_get::<i32, String>(2).unwrap(),
                                    label: v.raw_get::<i32, String>(3).unwrap(),
                                    value: v.raw_get::<i32, String>(4).unwrap(),
                                    id: i,
                                    entries: entries,
                                });

                                generator.lua.globals().set(k, control.clone()).unwrap();
                                new_controls.push(control);
                            }
                            "Checkbox" => {
                                let control = Control::CheckboxType(CheckboxInput {
                                    name: v.raw_get::<i32, String>(2).unwrap(),
                                    label: v.raw_get::<i32, String>(3).unwrap(),
                                    value: v.raw_get::<i32, i32>(4).unwrap() == 1,
                                });
                                new_controls.push(control);
                            }
                            "Label" => {
                                let control = Control::Label(v.raw_get::<i32, String>(2).unwrap());
                                new_controls.push(control);
                            }
                            "Separator" => {
                                let control = Control::Separator;
                                new_controls.push(control);
                            }
                            "Spacer" => {
                                let control = Control::Spacer;
                                new_controls.push(control);
                            }
                            _ => {}
                        }
                    }
                    Err(_) => {}
                }

                Ok(())
            }).expect("Failed to parse table");


            generator.controls.append(&mut new_controls);
        }
    }

    pub fn tick_lua(&self)
    {
        for generator in &self.generators
        {
            for control in &generator.controls {
                generator.lua.globals().set(control.get_name(), control.clone()).unwrap();
            }
            let tick: Function<'_> = generator.lua.globals().get("tick").unwrap();
            tick.call::<_, ()>(()).unwrap();
        }
    }

    pub fn render_controls(&mut self, ui: &mut Ui)
    {
        for generator in &mut self.generators {
            for control in &mut generator.controls {
                control.render(ui);
            }
        }
    }

    pub fn generate_includes(&mut self) -> String {
        let mut code: String = "".to_string();

        for generator in &self.generators {
            let tick: Function<'_> = generator.lua.globals().get("generate_includes").unwrap();
            code += &*tick.call::<_, String>(()).unwrap();
        }
        code.to_string()
    }

    pub fn generate_init(&mut self) -> String {
        let mut code: String = "".to_string();

        for generator in &self.generators {
            let tick: Function<'_> = generator.lua.globals().get("generate_init").unwrap();
            code += &*tick.call::<_, String>(()).unwrap();
        }
        code.to_string()
    }

    pub fn generate_globals(&mut self) -> String {
        let mut code: String = "".to_string();

        for generator in &self.generators {
            let tick: Function<'_> = generator.lua.globals().get("generate_globals").unwrap();
            code += &*tick.call::<_, String>(()).unwrap();
        }
        code.to_string()
    }
    pub fn generate_loop_one_time_setup(&mut self) -> String {
        let mut code: String = "".to_string();

        for generator in &self.generators {
            let tick: Function<'_> = generator.lua.globals().get("generate_loop_one_time_setup").unwrap();
            code += &*tick.call::<_, String>(()).unwrap();
        }
        code.to_string()
    }

    pub fn generate_loop(&mut self) -> String {
        let mut code: String = "".to_string();

        for generator in &self.generators {
            let tick: Function<'_> = generator.lua.globals().get("generate_loop").unwrap();
            code += &*tick.call::<_, String>(()).unwrap();
        }
        code.to_string()
    }
}

impl UserData for LuaGenerator {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(_methods: &mut M) {
        /*methods.add_method_mut("add_slider", |_, this, (min, max, value): (f32, f32, f32)| {
            println!("Adding slider");
            this.controls.push(Control::SliderType(Slider {
                min: min,
                max: max,
                value: value,
                step_by: 1.0,
                deicimals: 0,
                label: "".to_string(),
                keybinding: None,
            }));
            Ok(())
        });*/
    }
}

impl LuaGenerator {
    pub fn new(script_path: &str) -> Self {
        let mut generator = Self {
            lua: Lua::new(),
            script: script_path.to_string(),
            script_data: "".into(),
            loaded: false,
            controls: vec![],
        };
        generator.render();
        generator
    }

    pub fn load(&mut self) {
        if let true = self.loaded {
            return;
        }

        self.loaded = true;

        let mut script_data: String = "".to_string();

        let file = File::open(self.script.clone());

        file.unwrap().read_to_string(&mut script_data).unwrap();

        self.script_data = script_data.to_string();

        self.lua.load(&self.script_data).exec().unwrap();
    }
    pub fn render(&mut self)
    {
        self.load();
        return;
    }
}