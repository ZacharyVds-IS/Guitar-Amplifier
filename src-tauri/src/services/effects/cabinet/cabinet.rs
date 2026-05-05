use crate::domain::audio_processor::AudioProcessor;
use crate::domain::dto::effect::cabinet_dto::CabinetDto;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::effect::Effect;
use hound::{SampleFormat, WavReader};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::warn;

const DEFAULT_IR_FILE: &str = "reverb_oxford_lean.wav";

pub struct Cabinet {
	id: u32,
	name: String,
	is_active: Arc<AtomicBool>,
	color: String,
}

impl Cabinet {
	pub fn new(id: u32, name: String, is_active: bool, color: String) -> Self {
		if let Err(err) = Self::read_and_print_default_ir_buffer(DEFAULT_IR_FILE) {
			warn!("Cabinet IR debug read failed during creation: {err}");
		}

		Self {
			id,
			name,
			is_active: Arc::new(AtomicBool::new(is_active)),
			color,
		}
	}

	pub fn read_and_print_default_ir_buffer(file_name: &str) -> Result<Vec<f32>, String> {
		let ir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("resources")
			.join("default_ir")
			.join(file_name);

		let mut reader = WavReader::open(&ir_path)
			.map_err(|e| format!("Failed to open IR file '{}': {e}", ir_path.display()))?;

		let spec = reader.spec();
		let buffer = match spec.sample_format {
			SampleFormat::Float => reader
				.samples::<f32>()
				.collect::<Result<Vec<_>, _>>()
				.map_err(|e| format!("Failed to read float samples from '{}': {e}", ir_path.display()))?,
			SampleFormat::Int => {
				let max = ((1_i64 << (spec.bits_per_sample.saturating_sub(1))) - 1) as f32;
				reader
					.samples::<i32>()
					.map(|sample| sample.map(|value| value as f32 / max.max(1.0)))
					.collect::<Result<Vec<_>, _>>()
					.map_err(|e| format!("Failed to read int samples from '{}': {e}", ir_path.display()))?
			}
		};

		println!(
			"Loaded IR '{}' (channels={}, sample_rate={}, samples={})",
			ir_path.display(),
			spec.channels,
			spec.sample_rate,
			buffer.len()
		);
		println!("IR buffer: {:?}", buffer);

		Ok(buffer)
	}
}

impl AudioProcessor for Cabinet {
	fn process(&mut self, sample: f32) -> f32 {
		sample
	}
}

impl Effect for Cabinet {
	fn id(&self) -> u32 {
		self.id
	}

	fn name(&self) -> &str {
		&self.name
	}

	fn get_color(&self) -> String {
		self.color.clone()
	}

	fn to_dto(&self) -> EffectDto {
		EffectDto::Cabinet(CabinetDto {
			id: self.id,
			name: self.name.clone(),
			is_active: self.is_active.load(Ordering::Relaxed),
			color: self.color.clone(),
		})
	}

	fn active_flag(&self) -> Arc<AtomicBool> {
		Arc::clone(&self.is_active)
	}
}

