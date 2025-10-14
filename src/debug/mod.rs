use crate::apu::Apu;
use crate::emulator::NTSC_CPU_FREQUENCY;
use imgui::{TreeNodeFlags, Ui};

const HISTORY_SIZE: usize = 200;

macro_rules! text_boolean {
    ($ui:expr, $value:expr, $true_text:expr, $false_text:expr) => {
        $ui.text_colored(
            if $value { [0.0, 1.0, 0.0, 1.0] } else { [1.0, 0.0, 0.0, 1.0] },
            if $value { $true_text } else { $false_text },
        )
    };
}

pub struct ApuDebugPanel {
    output_waveform: Vec<f32>,
    channel_waveforms: [Vec<f32>; 5],

    output_peak_level: f32,
    channel_peak_levels: [f32; 5],
}

impl Default for ApuDebugPanel {
    fn default() -> Self {
        Self {
            output_waveform: vec![0.0; HISTORY_SIZE],
            channel_waveforms: [
                vec![0.0; HISTORY_SIZE],
                vec![0.0; HISTORY_SIZE],
                vec![0.0; HISTORY_SIZE],
                vec![0.0; HISTORY_SIZE],
                vec![0.0; HISTORY_SIZE],
            ],
            output_peak_level: 0.0,
            channel_peak_levels: [0.0; 5],
        }
    }
}

impl ApuDebugPanel {
    pub fn update(&mut self, apu: &Apu, output: f32) {
        self.output_waveform.rotate_left(1);
        if let Some(last) = self.output_waveform.last_mut() {
            *last = output;
        }

        let current_level = output.abs();
        if current_level > self.output_peak_level {
            self.output_peak_level = current_level;
        } else {
            self.output_peak_level *= 0.95; // Decay over time for better visualisation
        }

        let channel_outputs = [
            apu.pulse_1.raw_output() as f32 / 15.0,
            apu.pulse_2.raw_output() as f32 / 15.0,
            apu.triangle.raw_output() as f32 / 15.0,
            apu.noise.raw_output() as f32 / 15.0,
            apu.dmc.raw_output() as f32 / 127.0,
        ];

        for (i, channel_output) in channel_outputs.iter().enumerate() {
            self.channel_waveforms[i].rotate_left(1);
            if let Some(last) = self.channel_waveforms[i].last_mut() {
                *last = *channel_output;
            }

            let current_level = channel_output.abs();
            if current_level > self.channel_peak_levels[i] {
                self.channel_peak_levels[i] = current_level;
            } else {
                self.channel_peak_levels[i] *= 0.95; // Decay over time for better visualisation
            }
        }
    }

    pub fn render(&mut self, ui: &Ui, apu: &mut Apu) {
        if !ui.collapsing_header("APU Debug", TreeNodeFlags::DEFAULT_OPEN) {
            return;
        }

        if let Some(_tab_bar) = ui.tab_bar("apu_tabs") {
            if let Some(_tab) = ui.tab_item("Channels") {
                self.render_channel_status(ui, apu);
            }

            if let Some(_tab) = ui.tab_item("Visualisation") {
                self.render_visualisation(ui);
            }

            if let Some(_tab) = ui.tab_item("Audio Controls") {
                self.render_audio_controls(ui, apu);
            }
        }
    }

    fn render_channel_status(&self, ui: &Ui, apu: &Apu) {
        ui.text("Channel Status");
        ui.separator();

        ui.tree_node_config("Pulse 1").default_open(true).build(|| {
            self.render_pulse_channel_status(ui, apu, 1);
        });

        ui.tree_node_config("Pulse 2").default_open(true).build(|| {
            self.render_pulse_channel_status(ui, apu, 2);
        });

        ui.tree_node_config("Triangle").default_open(true).build(|| {
            self.render_triangle_channel_status(ui, apu);
        });

        ui.tree_node_config("Noise").default_open(true).build(|| {
            self.render_noise_channel_status(ui, apu);
        });

        ui.tree_node_config("DMC").default_open(true).build(|| {
            self.render_dmc_channel_status(ui, apu);
        });
    }

    fn render_pulse_channel_status(&self, ui: &Ui, apu: &Apu, channel: usize) {
        let (prefix, enabled, length, timer, output) = if channel == 1 {
            (
                "pulse1",
                apu.pulse_1.enabled,
                apu.pulse_1.length_counter,
                apu.pulse_1.timer_period,
                apu.pulse_1.raw_output(),
            )
        } else {
            (
                "pulse2",
                apu.pulse_2.enabled,
                apu.pulse_2.length_counter,
                apu.pulse_2.timer_period,
                apu.pulse_2.raw_output(),
            )
        };

        ui.columns(2, format!("{prefix}_cols"), true);

        ui.text("Enabled:");
        ui.next_column();
        text_boolean!(ui, enabled, "Yes", "No");
        ui.next_column();

        ui.text("Length Counter:");
        ui.next_column();
        ui.text(format!("{}", length));
        ui.next_column();

        ui.text("Timer Period:");
        ui.next_column();
        ui.text(format!(
            "{} ({:.1} Hz)",
            timer,
            if timer > 0 { NTSC_CPU_FREQUENCY / (16.0 * (timer as f64 + 1.0)) } else { 0.0 }
        ));
        ui.next_column();

        ui.text("Output:");
        ui.next_column();
        self.render_value_bar(ui, output as f32 / 15.0);
        ui.next_column();

        ui.columns(1, "", false);
    }

    fn render_triangle_channel_status(&self, ui: &Ui, apu: &Apu) {
        ui.columns(2, "triangle_cols", true);

        ui.text("Enabled:");
        ui.next_column();
        text_boolean!(ui, apu.triangle.enabled, "Yes", "No");
        ui.next_column();

        ui.text("Length Counter:");
        ui.next_column();
        ui.text(format!("{}", apu.triangle.length_counter));
        ui.next_column();

        ui.text("Timer Period:");
        ui.next_column();
        ui.text(format!(
            "{} ({:.1} Hz)",
            apu.triangle.timer_period,
            if apu.triangle.timer_period > 0 {
                NTSC_CPU_FREQUENCY / (32.0 * (apu.triangle.timer_period as f64 + 1.0))
            } else {
                0.0
            }
        ));
        ui.next_column();

        ui.text("Linear Counter:");
        ui.next_column();
        ui.text(format!("{}", apu.triangle.linear_counter));
        ui.next_column();

        ui.text("Output:");
        ui.next_column();
        self.render_value_bar(ui, apu.triangle.raw_output() as f32 / 15.0);
        ui.next_column();

        ui.columns(1, "", false);
    }

    fn render_noise_channel_status(&self, ui: &Ui, apu: &Apu) {
        ui.columns(2, "noise_cols", true);

        ui.text("Enabled:");
        ui.next_column();
        text_boolean!(ui, apu.noise.enabled, "Yes", "No");
        ui.next_column();

        ui.text("Length Counter:");
        ui.next_column();
        ui.text(format!("{}", apu.noise.length_counter));
        ui.next_column();

        ui.text("Timer Period:");
        ui.next_column();
        ui.text(format!(
            "{} ({:.1} Hz)",
            apu.noise.timer_period,
            if apu.noise.timer_period > 0 {
                NTSC_CPU_FREQUENCY / (apu.noise.timer_period as f64 * 2.0)
            } else {
                0.0
            }
        ));
        ui.next_column();

        ui.text("Mode:");
        ui.next_column();
        ui.text(if apu.noise.mode { "Normal" } else { "Short" });
        ui.next_column();

        ui.text("Output:");
        ui.next_column();
        self.render_value_bar(ui, apu.noise.raw_output() as f32 / 15.0);
        ui.next_column();

        ui.columns(1, "", false);
    }

    fn render_dmc_channel_status(&self, ui: &Ui, apu: &Apu) {
        ui.columns(2, "dmc_cols", true);

        ui.text("Enabled:");
        ui.next_column();
        text_boolean!(ui, apu.dmc.enabled, "Yes", "No");
        ui.next_column();

        ui.text("Bytes Remaining:");
        ui.next_column();
        ui.text(format!("{}", apu.dmc.bytes_remaining));
        ui.next_column();

        ui.text("Rate Index:");
        ui.next_column();
        ui.text(format!("{}", apu.dmc.rate_index));
        ui.next_column();

        ui.text("Sample Rate:");
        ui.next_column();
        ui.text(format!(
            "{} ({:.1} Hz)",
            apu.dmc.timer_period,
            if apu.dmc.timer_period > 0 {
                NTSC_CPU_FREQUENCY / apu.dmc.timer_period as f64
            } else {
                0.0
            }
        ));
        ui.next_column();

        ui.text("Sample Address:");
        ui.next_column();
        ui.text(format!("${:04X}", apu.dmc.sample_address));
        ui.next_column();

        ui.text("Output:");
        ui.next_column();
        self.render_value_bar(ui, apu.dmc.raw_output() as f32 / 127.0);
        ui.next_column();

        ui.columns(1, "", false);
    }

    fn render_value_bar(&self, ui: &Ui, value: f32) {
        let width = ui.content_region_avail()[0];
        let height = 20.0;

        let draw_list = ui.get_window_draw_list();
        let pos = ui.cursor_screen_pos();

        draw_list.add_rect(pos, [pos[0] + width, pos[1] + height], [0.3, 0.3, 0.3]).filled(true).build();

        let bar_width = width * value.clamp(0.0, 1.0);
        let colour = self.value_to_colour(value);
        draw_list.add_rect(pos, [pos[0] + bar_width, pos[1] + height], colour).filled(true).build();

        let text = format!("{:.1}%", value * 100.0);
        let text_colour = self.contrasting_colour(colour);
        draw_list.add_text([pos[0] + 5.0, pos[1] + 3.0], text_colour, text);

        ui.dummy([width, height]);
    }

    fn value_to_colour(&self, value: f32) -> [f32; 4] {
        if value < 0.5 {
            [value * 2.0, 1.0, 0.0, 1.0]
        } else {
            [1.0, 1.0 - (value - 0.5) * 2.0, 0.0, 1.0]
        }
    }

    fn contrasting_colour(&self, background: [f32; 4]) -> [f32; 4] {
        let luminance = 0.2126 * background[0] + 0.7152 * background[1] + 0.0722 * background[2];
        if luminance > 0.5 {
            [0.0, 0.0, 0.0, 1.0] // Black text
        } else {
            [1.0, 1.0, 1.0, 1.0] // White text
        }
    }

    fn render_visualisation(&self, ui: &Ui) {
        ui.text("Level Meters");
        ui.separator();

        self.render_level_meters(ui);

        ui.spacing();

        ui.text("Output Waveform");
        ui.separator();

        if ui.collapsing_header("Master Output", TreeNodeFlags::DEFAULT_OPEN) {
            self.render_waveform(ui, &self.output_waveform, "master");
        }

        if ui.collapsing_header("Pulse 1", TreeNodeFlags::empty()) {
            self.render_waveform(ui, &self.channel_waveforms[0], "pulse1");
        }

        if ui.collapsing_header("Pulse 2", TreeNodeFlags::empty()) {
            self.render_waveform(ui, &self.channel_waveforms[1], "pulse2");
        }

        if ui.collapsing_header("Triangle", TreeNodeFlags::empty()) {
            self.render_waveform(ui, &self.channel_waveforms[2], "triangle");
        }

        if ui.collapsing_header("Noise", TreeNodeFlags::empty()) {
            self.render_waveform(ui, &self.channel_waveforms[3], "noise");
        }

        if ui.collapsing_header("DMC", TreeNodeFlags::empty()) {
            self.render_waveform(ui, &self.channel_waveforms[4], "dmc");
        }
    }

    fn render_level_meters(&self, ui: &Ui) {
        ui.columns(2, "level_meter_cols", false);

        ui.text("Master:");
        ui.next_column();
        self.render_value_bar(ui, self.output_peak_level);
        ui.next_column();

        let channels = [("Pulse 1", 0), ("Pulse 2", 1), ("Triangle", 2), ("Noise", 3), ("DMC", 4)];

        for (name, idx) in channels {
            let peak = self.channel_peak_levels[idx];

            ui.text(format!("{}:", name));
            ui.next_column();
            self.render_value_bar(ui, peak);
            ui.next_column();
        }

        ui.columns(1, "", false);
    }

    fn render_waveform(&self, ui: &Ui, waveform: &[f32], id: &str) {
        let size = [ui.content_region_avail()[0], 150.0];

        ui.plot_lines(id, waveform).graph_size(size).scale_min(-1.0).scale_max(1.0).build();

        let draw_list = ui.get_window_draw_list();
        let pos = ui.cursor_screen_pos();
        let center_y = pos[1] - size[1] / 2.0;
        draw_list
            .add_line([pos[0], center_y], [pos[0] + size[0], center_y], [0.5, 0.5, 0.5, 0.5])
            .build();
    }

    fn render_audio_controls(&self, ui: &Ui, apu: &mut Apu) {
        let audio_processor = &mut apu.audio_processor;

        ui.text("Master Volume:");
        ui.slider_config("##master_vol", 0.0, 2.0)
            .display_format("%.2f")
            .build(&mut audio_processor.master_volume);
        ui.same_line();
        if ui.button("Mute##master_vol") {
            audio_processor.master_volume = 0.0;
        }
        ui.same_line();
        if ui.button("Reset##master_vol") {
            audio_processor.master_volume = 1.0;
        }

        let channels = [
            ("Pulse 1", "pulse1", &mut audio_processor.channel_volumes.pulse_1),
            ("Pulse 2", "pulse2", &mut audio_processor.channel_volumes.pulse_2),
            ("Triangle", "triangle", &mut audio_processor.channel_volumes.triangle),
            ("Noise", "noise", &mut audio_processor.channel_volumes.noise),
            ("DMC", "dmc", &mut audio_processor.channel_volumes.dmc),
        ];

        for (label, prefix, volume) in channels {
            ui.text(label);
            ui.slider_config(format!("##{}_vol", prefix), 0.0, 2.0).display_format("%.2f").build(volume);
            ui.same_line();
            if ui.button(format!("Mute##{}_vol", prefix)) {
                *volume = 0.0;
            }
            ui.same_line();
            if ui.button(format!("Reset##{}_vol", prefix)) {
                *volume = 1.0;
            }
        }
    }
}
