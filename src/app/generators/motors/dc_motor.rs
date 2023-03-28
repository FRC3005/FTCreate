use egui::{widgets::ImageButton, Vec2};
use egui_extras::RetainedImage;

use serde::{Deserialize, Serialize};

use strum::IntoEnumIterator;

use super::super::generator;
use super::motor;

use crate::app::generators::{
    self,
    generator::GeneratorSerialize,
    keybinding::keybinding::{BooleanButton, Keybinding},
    method::Method,
    motors,
    subsystem::subsystem::DrivetrainType,
};

use motor::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct DcMotor {
    pub direction: motors::motor::MotorDirection,
    pub mode: motors::motor::MotorMode,
    pub max_speed: f64,
    pub mecanum_position: motors::motor::MecanumPosition,
    pub arcade_position: motors::motor::ArcadePosition,
    pub name: String,
    pub positions: Vec<Keybinding>,
    pub drivetrain_type: Option<DrivetrainType>,
}

impl DcMotor {}

impl GeneratorSerialize for DcMotor {}

impl generator::Generator for DcMotor {
    fn get_methods(&self) -> Vec<Method> {
        vec![Method {
            name: "setPower".to_string(),
            num_args: 1,
        }]
    }

    fn generate_includes(&self) -> String {
        "\
        import com.qualcomm.robotcore.hardware.DcMotor;\n\
        import org.firstinspires.ftc.robotcore.external.Telemetry;\n\
        import com.qualcomm.robotcore.hardware.HardwareMap;\n\
        import com.qualcomm.robotcore.hardware.DcMotorEx;\n\n"
            .to_string()
    }

    fn generate_globals(&self) -> String {
        let mut code = format!(
            "\t// {} globals\n\tprivate DcMotorEx {} = null;\n\n",
            &self.name, &self.name
        );

        if self.mode == MotorMode::RUN_TO_POSITION {
            for i in 0..self.positions.len() {
                code += &format!(
                    "\tprivate int {}_pos_{} = {};\n",
                    self.name,
                    i,
                    self.positions.iter().nth(i).unwrap().value
                );
            }
            code += &"\n";
        }

        code
    }

    fn generate_init(&self) -> String {
        format!(
            "\t\t// {} init\n\t\t{} = hardwareMap.get(DcMotorEx.class, \"{}\");\n\n",
            &self.name, &self.name, &self.name
        ) + &format!(
            "\t\t{}.setDirection(DcMotor.Direction.{:?});\n\n",
            &self.name, &self.direction
        ) + &format!(
            "\t\t{}.setMode(DcMotor.RunMode.STOP_AND_RESET_ENCODER);\n",
            &self.name
        ) + &format!(
            "\t\t{}.setMode(DcMotor.RunMode.{:?});\n\n",
            &self.name, &self.mode
        )
    }

    fn generate_loop(&self) -> String {
        let mut code = match self.drivetrain_type {
        Some(DrivetrainType::Mecanum) => match self.mecanum_position {
            MecanumPosition::FrontLeft =>
                format!(
                    "\t\t\t// {} loop\n\t\t\t{}.setPower(Range.clip(drive - strafe + turn, -{}, {}));\n\n",
                    &self.name, &self.name, self.max_speed, self.max_speed
                ),
            MecanumPosition::FrontRight =>
                format!(
                    "\t\t\t// {} loop\n\t\t\t{}.setPower(Range.clip(drive + strafe - turn, -{}, {}));\n\n",
                    &self.name, &self.name, self.max_speed, self.max_speed
                ),
            MecanumPosition::RearLeft =>
                format!(
                    "\t\t\t// {} loop\n\t\t\t{}.setPower(Range.clip(drive + strafe + turn, -{}, {}));\n\n",
                    &self.name, &self.name, self.max_speed, self.max_speed
                ),
            MecanumPosition::RearRight =>
                format!(
                    "\t\t\t// {} loop\n\t\t\t{}.setPower(Range.clip(drive - strafe - turn, -{}, {}));\n\n",
                    &self.name, &self.name, self.max_speed, self.max_speed
                ),
        },
            Some(DrivetrainType::Arcade) => {
                match self.arcade_position {
                    ArcadePosition::Left => {
                format!(
                    "\t\t\t// {} loop\n\t\t\t{}.setPower(Range.clip(drive + turn, -{}, {}));\n\n",
                    &self.name, &self.name, self.max_speed, self.max_speed
                )},
                    ArcadePosition::Right => {
                format!(
                    "\t\t\t// {} loop\n\t\t\t{}.setPower(Range.clip(drive - turn, -{}, {}));\n\n",
                    &self.name, &self.name, self.max_speed, self.max_speed
                )},
                    }
                },
            Some(DrivetrainType::Tank) => {
                match self.arcade_position {
                    ArcadePosition::Left => {
                format!(
                    "\t\t\t// {} loop\n\t\t\t{}.setPower(Range.clip(driveLeft, -{}, {}));\n\n",
                    &self.name, &self.name, self.max_speed, self.max_speed
                )},
                    ArcadePosition::Right => {
                format!(
                    "\t\t\t// {} loop\n\t\t\t{}.setPower(Range.clip(driveRight, -{}, {}));\n\n",
                    &self.name, &self.name, self.max_speed, self.max_speed
                )},
                    }
                }
            None => "".to_string(),
    };
        // generate keybindings
        for i in 0..self.positions.len() {
            if self.positions.iter().nth(i).unwrap().button != None {
                code += &format!(
                    "\t\t\tif (driveController.buttonJustPressed({:?}) {{\n",
                    &self.positions.iter().nth(i).unwrap().button.unwrap()
                );

                code += &format!(
                    "\t\t\t\t{}.runToPosition({});\n",
                    self.name,
                    self.positions.iter().nth(i).unwrap().value
                );

                code += &"\t\t\t}\n";
            }
        }
        code
    }

    fn render_options(&mut self, ui: &mut egui::Ui, id: usize) {
        let max_speed = 1.0;

        ui.label("DC Motor");
        ui.add_space(10.0);

        ui.text_edit_singleline(&mut self.name);

        ui.push_id(id, |ui| {
            egui::ComboBox::from_label("Run mode")
                .selected_text(format!("{:?}", &mut self.mode))
                .width(170.0)
                .show_ui(ui, |ui| {
                    for mode in motor::MotorMode::iter() {
                        ui.selectable_value(&mut self.mode, mode, format!("{:?}", mode));
                    }
                });
        });

        ui.push_id(id, |ui| {
            egui::ComboBox::from_label("Direction")
                .selected_text(format!("{:?}", &mut self.direction))
                .width(170.0)
                .show_ui(ui, |ui| {
                    for direction in motor::MotorDirection::iter() {
                        ui.selectable_value(
                            &mut self.direction,
                            direction,
                            format!("{:?}", direction),
                        );
                    }
                });
        });

        ui.add_space(20.0);

        ui.add(
            egui::Slider::new(&mut self.max_speed, 0.0..=max_speed)
                .text("Max speed")
                .max_decimals(2),
        );

        ui.add_space(20.0);

        if let Some(drivetrain_type) = self.drivetrain_type {
            if drivetrain_type == DrivetrainType::Mecanum {
                self.render_mecanum(ui, id);
            } else {
                self.render_arcade(ui, id);
            }
        }

        if self.mode == MotorMode::RUN_TO_POSITION {
            self.render_positions(ui, id);
        }
    }
}

impl DcMotor {
    fn render_positions(&mut self, ui: &mut egui::Ui, _id: usize) {
        ui.add_space(10.0);
        ui.label("Fixed positions");

        let mut removed_positions = vec![];

        for (i, pos) in self.positions.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut pos.value, 0..=5000)
                        .text("Position")
                        .step_by(1.0)
                        .max_decimals(2),
                );

                if ui.button("Delete").clicked() {
                    removed_positions.push(i);
                }

                let image = RetainedImage::from_image_bytes(
                    "gamepad.png".to_string(),
                    crate::app::generators::keybinding::keybinding::GAMEPAD_IMAGE,
                )
                .unwrap();

                let button = ImageButton::new(
                    image.texture_id(ui.ctx()),
                    Vec2 {
                        x: 16.0,
                        y: 16.0 * 0.774,
                    },
                );
                if ui.add(button).clicked() {
                    println!("Keybinding button!");
                    pos.button = Some(BooleanButton::A);
                    //self.keybindings.push("Test keybinding".to_string());
                }
            });
        }

        for i in removed_positions {
            self.positions.remove(i);
        }

        ui.horizontal(|ui| {
            if ui.button("Add position").clicked() {
                self.positions.push(Keybinding::new(0));
            }
        });
    }

    fn render_mecanum(&mut self, ui: &mut egui::Ui, id: usize) {
        ui.push_id(id + 100, |ui| {
            egui::ComboBox::from_label("Mecanum position")
                .selected_text(format!("{:?}", &mut self.mecanum_position))
                .width(170.0)
                .show_ui(ui, |ui| {
                    for position in MecanumPosition::iter() {
                        ui.selectable_value(
                            &mut self.mecanum_position,
                            position,
                            format!("{:?}", position),
                        );
                    }
                });
        });
    }

    fn render_arcade(&mut self, ui: &mut egui::Ui, id: usize) {
        ui.push_id(id + 100, |ui| {
            egui::ComboBox::from_label(format!("{:?} position", self.drivetrain_type.unwrap()))
                .selected_text(format!("{:?}", &mut self.arcade_position))
                .width(170.0)
                .show_ui(ui, |ui| {
                    for position in ArcadePosition::iter() {
                        ui.selectable_value(
                            &mut self.arcade_position,
                            position,
                            format!("{:?}", position),
                        );
                    }
                });
        });
    }
}

impl Motor for DcMotor {}

impl MotorGenerator for DcMotor {
    fn new(name: String) -> Self {
        DcMotor {
            direction: generators::motors::motor::MotorDirection::FORWARD,
            mode: generators::motors::motor::MotorMode::RUN_TO_POSITION,
            max_speed: 1.0,
            mecanum_position: MecanumPosition::FrontLeft,
            arcade_position: ArcadePosition::Left,
            name: name,
            positions: vec![],
            drivetrain_type: None,
        }
    }

    fn set_drivetrain_type(&mut self, drivetrain_type: Option<DrivetrainType>) {
        self.drivetrain_type = drivetrain_type;
    }

    fn set_mecanum_position(&mut self, position: MecanumPosition) {
        self.mecanum_position = position;
    }
}
