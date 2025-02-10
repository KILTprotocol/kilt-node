// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

// The KILT Blockchain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The KILT Blockchain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// If you feel like getting in touch with us, you can do so at <hello@kilt.org>

use substrate_wasm_builder::WasmBuilder;

fn main() {
	let builder = WasmBuilder::new()
		.with_current_project()
		.export_heap_base()
		.import_memory();

	#[cfg(feature = "metadata-hash")]
	// TODO: Can we re-use some consts?
	let builder = builder.enable_metadata_hash("PILT", 15);

	builder.build()
}
