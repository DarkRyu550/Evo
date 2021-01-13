use crate::state::State;
use std::time::Duration;
use std::borrow::Borrow;
use wgpu::{Surface, SwapChain, SwapChainDescriptor, TextureUsage, TextureFormat, PresentMode, SwapChainError, CommandEncoderDescriptor, RenderPassDescriptor, RenderPassColorAttachmentDescriptor, RenderPipeline, RenderPipelineDescriptor, PipelineLayout, PipelineLayoutDescriptor, BindGroupLayoutDescriptor, BindGroupEntry, ProgrammableStageDescriptor, ShaderModule, BindGroupLayout, BindGroup, Device, RasterizationStateDescriptor, FrontFace, CullMode, PrimitiveTopology, ColorStateDescriptor, VertexStateDescriptor, VertexBufferDescriptor, IndexFormat, InputStepMode, VertexAttributeDescriptor, VertexFormat, ShaderModuleSource, Buffer, BindGroupLayoutEntry, ShaderStage, BindingType, BufferUsage, BindGroupDescriptor, BindingResource, BufferSlice, Operations, Color, LoadOp, CommandEncoder, Texture, TextureDescriptor, Extent3d, TextureDimension, TextureViewDescriptor, TextureViewDimension, TextureAspect, TextureView};
use crate::settings::{Preferences, PresentationMode};
use crate::flipbook::Consumer;
use wgpu::util::{DeviceExt, BufferInitDescriptor};
use crate::dataset::Matrix4;
use std::convert::TryFrom;

/** Shaders and pipeline bundle for geometry passes. */
struct GeometryPipeline {
	/** Handle to the vertex shader. */
	pub vertex: ShaderModule,
	/** Handle to the fragment shader. */
	pub fragment: ShaderModule,
	/** Handle to the layout of the rendering pipeline. */
	pub layout: PipelineLayout,
	/** Handle to the pipeline. */
	pub pipeline: RenderPipeline
}
impl GeometryPipeline {
	/** Create a new pipeline from the given shaders. */
	pub fn new(
		device:   &Device,
		flipbook: &Consumer,
		label:    Option<&'static str>,
		graphics: &RenderParameters,
		vertex:   ShaderModuleSource<'_>,
		fragment: ShaderModuleSource<'_>) -> Self {

		let vertex   = device.create_shader_module(vertex);
		let fragment = device.create_shader_module(fragment);
		let layout = device.create_pipeline_layout(
			&PipelineLayoutDescriptor {
				label,
				bind_group_layouts: &[
					flipbook.layout(),
					&graphics.layout
				],
				push_constant_ranges: &[]
			});
		let pipeline = device.create_render_pipeline(
			&RenderPipelineDescriptor {
				label,
				layout: Some(&layout),
				vertex_stage: ProgrammableStageDescriptor {
					module: &vertex,
					entry_point: "main"
				},
				fragment_stage: Some(ProgrammableStageDescriptor {
					module: &fragment,
					entry_point: "main"
				}),
				rasterization_state: Some(RasterizationStateDescriptor {
					front_face: FrontFace::Ccw,
					cull_mode: CullMode::None,
					clamp_depth: false,
					depth_bias: 0,
					depth_bias_slope_scale: 0.0,
					depth_bias_clamp: 0.0
				}),
				primitive_topology: PrimitiveTopology::TriangleStrip,
				color_states: &[
					ColorStateDescriptor {
						format: TextureFormat::Rgba32Float,
						alpha_blend: Default::default(),
						color_blend: Default::default(),
						write_mask: Default::default()
					},
					ColorStateDescriptor {
						format: TextureFormat::Rgba32Float,
						alpha_blend: Default::default(),
						color_blend: Default::default(),
						write_mask: Default::default()
					}
				],
				depth_stencil_state: None,
				vertex_state: VertexStateDescriptor {
					index_format: IndexFormat::Uint32,
					vertex_buffers: &[
						VertexBufferDescriptor {
							stride: 24,
							step_mode: InputStepMode::Vertex,
							attributes: &[
								/* Position attribute. */
								VertexAttributeDescriptor {
									offset: 0,
									format: VertexFormat::Float3,
									shader_location: 0
								},
								/* Vertex normal attribute. */
								VertexAttributeDescriptor {
									offset: 12,
									format: VertexFormat::Float3,
									shader_location: 1
								}
							]
						}
					]
				},
				sample_count: 1,
				sample_mask: !0,
				alpha_to_coverage_enabled: false
			});

		Self {
			vertex,
			fragment,
			layout,
			pipeline
		}
	}
}

/** Shaders and pipeline bundle for the lighting pass. */
struct LightingPipeline {
	/** Handle to the vertex shader. */
	pub vertex: ShaderModule,
	/** Handle to the fragment shader. */
	pub fragment: ShaderModule,
	/** Handle to the layout of the rendering pipeline. */
	pub layout: PipelineLayout,
	/** Handle to the pipeline. */
	pub pipeline: RenderPipeline,
}
impl LightingPipeline {
	/** Create a new pipeline from the given shaders. */
	pub fn new(
		device:   &Device,
		label:    Option<&'static str>,
		geometry: &Geometry,
		vertex:   ShaderModuleSource<'_>,
		fragment: ShaderModuleSource<'_>) -> Self {

		let vertex   = device.create_shader_module(vertex);
		let fragment = device.create_shader_module(fragment);
		let layout = device.create_pipeline_layout(
			&PipelineLayoutDescriptor {
				label,
				bind_group_layouts: &[
					&geometry.layout
				],
				push_constant_ranges: &[]
			});
		let pipeline = device.create_render_pipeline(
			&RenderPipelineDescriptor {
				label,
				layout: Some(&layout),
				vertex_stage: ProgrammableStageDescriptor {
					module: &vertex,
					entry_point: "main"
				},
				fragment_stage: Some(ProgrammableStageDescriptor {
					module: &fragment,
					entry_point: "main"
				}),
				rasterization_state: Some(RasterizationStateDescriptor {
					front_face: FrontFace::Ccw,
					cull_mode: CullMode::None,
					clamp_depth: false,
					depth_bias: 0,
					depth_bias_slope_scale: 0.0,
					depth_bias_clamp: 0.0
				}),
				primitive_topology: PrimitiveTopology::TriangleStrip,
				color_states: &[
					ColorStateDescriptor {
						format: TextureFormat::Bgra8Unorm,
						alpha_blend: Default::default(),
						color_blend: Default::default(),
						write_mask: Default::default()
					},
				],
				depth_stencil_state: None,
				vertex_state: VertexStateDescriptor {
					index_format: IndexFormat::Uint32,
					vertex_buffers: &[
						VertexBufferDescriptor {
							stride: 24,
							step_mode: InputStepMode::Vertex,
							attributes: &[
								/* Position attribute. */
								VertexAttributeDescriptor {
									offset: 0,
									format: VertexFormat::Float3,
									shader_location: 0
								},
								/* Vertex normal attribute. */
								VertexAttributeDescriptor {
									offset: 12,
									format: VertexFormat::Float3,
									shader_location: 1
								}
							]
						}
					]
				},
				sample_count: 1,
				sample_mask: !0,
				alpha_to_coverage_enabled: false
			});

		Self {
			vertex,
			fragment,
			layout,
			pipeline
		}
	}
}

/** Render parameter bundle. */
struct RenderParameters {
	/** Parameter uniform layout. */
	layout: BindGroupLayout,
	/** Backing uniform buffer. */
	buffer: Buffer,
	/** Parameter uniform bind group. */
	bind: BindGroup,
}
impl RenderParameters {
	fn new(device: &Device, params: crate::dataset::RenderParameters) -> Self {
		let layout = device.create_bind_group_layout(
			&BindGroupLayoutDescriptor {
				label: Some("Display/RenderParameters/Layout"),
				entries: &[
					BindGroupLayoutEntry {
						binding: 0,
						visibility: ShaderStage::VERTEX,
						ty: BindingType::UniformBuffer {
							dynamic: false,
							min_binding_size: None
						},
						count: None
					}
				]
			});

		let mut buffer = Vec::with_capacity(32);
		params.bytes(&mut buffer);

		let buffer = device.create_buffer_init(
			&BufferInitDescriptor {
				label: Some("Display/RenderParameters/Buffer"),
				contents: &buffer[..],
				usage: BufferUsage::UNIFORM
			});

		let bind = device.create_bind_group(
			&BindGroupDescriptor {
				label: Some("Display/RenderParameters/Bind"),
				layout: &layout,
				entries: &[
					BindGroupEntry {
						binding: 0,
						resource: BindingResource::Buffer(buffer.slice(..))
					}
				]
			});

		Self {
			layout,
			buffer,
			bind
		}
	}
}

/** Model data, already in GPU memory and ready to be used. */
struct Model {
	buffer: Buffer,
	index_count: u32,
	index: u64,
}
impl Model {
	/** Create a new model from the given data parameters. */
	pub fn new(device: &Device, indices: &[u8], vertices: &[u8]) -> Self {
		let index_count = u32::try_from(indices.len() / 4)
			.expect("cannot count indices");
		let index = u64::try_from(vertices.len())
			.expect("cannot represent u64 from usize");

		let mut buf = Vec::with_capacity(indices.len() + vertices.len());
		buf.extend_from_slice(vertices);
		buf.extend_from_slice(indices);

		let buffer = device.create_buffer_init(
			&BufferInitDescriptor {
				label: Some("Display/Model"),
				contents: &buf[..],
				usage: BufferUsage::VERTEX | BufferUsage::INDEX
			});

		Self { buffer, index_count, index }
	}

	/** Buffer slice corresponding to the index data set. */
	pub fn indices(&self) -> BufferSlice {
		self.buffer.slice(self.index..)
	}

	/** Number of indices in the model. */
	pub fn index_count(&self) -> u32 {
		self.index_count
	}

	/** Buffer slice corresponding to the vertex data set. */
	pub fn vertices(&self) -> BufferSlice {
		self.buffer.slice(..self.index)
	}
}

/** Screen-space geometry buffers used throughout the rendering process. */
struct Geometry {
	/** Width of the frames in this buffer. */
	pub width: u32,
	/** Height of the frames in this buffer. */
	pub height: u32,

	/** Albedo and specular components of the geometry. (`Rgba32Float`)
	 *
	 * The albedo value is stored as an `Rgb24Float` value in the first three
	 * components, while the specular value is stored in the fourth component,
	 * as an `R8Float` value. */
	pub albedo_specular: Texture,

	/** Normal and shadow components of the geometry. (`Rgba32Float`)
	 *
	 * The normal vector is stored as an `Rgb24Float` value in the first three
	 * components, while the shadow value is stored in the fourth component, as
	 * an `R8Float` value. */
	pub normal_shadow: Texture,

	/** Layout of the binding group of these resources. */
	pub layout: BindGroupLayout,
	/** Binding group of these resources. */
	pub bind: BindGroup
}
impl Geometry {
	/** Creates a new geometry buffer set from the given screen size. */
	pub fn new(device: &Device, width: u32, height: u32) -> Self {
		/* Create the textures. */
		let texture_base = TextureDescriptor {
			label: None,
			size: Extent3d { width, height, depth: 1 },
			mip_level_count: 1,
			sample_count: 1,
			dimension: TextureDimension::D2,
			format: TextureFormat::Rgba32Float,
			usage: TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::STORAGE
		};
		let view_base = TextureViewDescriptor {
			label: None,
			format: Some(TextureFormat::Rgba32Float),
			dimension: Some(TextureViewDimension::D2),
			aspect: TextureAspect::All,
			base_mip_level: 0,
			level_count: None,
			base_array_layer: 0,
			array_layer_count: None
		};

		let label = format!("Display/Geometry[{}x{}]/AlbedoSpecular",
			width, height);
		let albedo_specular = device.create_texture(
			&TextureDescriptor {
				label: Some(&label),
				..texture_base
			});
		let albedo_specular_view = albedo_specular.create_view(
			&TextureViewDescriptor {
				label: Some(&label),
				..view_base
			});

		let label = format!("Display/Geometry[{}x{}]/NormalShadow",
			width, height);
		let normal_shadow = device.create_texture(
			&TextureDescriptor {
				label: Some(&label),
				..texture_base
			});
		let normal_shadow_view = normal_shadow.create_view(
			&TextureViewDescriptor {
				label: Some(&label),
				..view_base
			});

		let layout = device.create_bind_group_layout(
			&BindGroupLayoutDescriptor {
				label: Some("Display/Geometry/Layout"),
				entries: &[
					BindGroupLayoutEntry {
						binding: 0,
						visibility: ShaderStage::FRAGMENT,
						ty: BindingType::StorageTexture {
							dimension: TextureViewDimension::D2,
							format: TextureFormat::Rgba32Float,
							readonly: true
						},
						count: None
					},
					BindGroupLayoutEntry {
						binding: 1,
						visibility: ShaderStage::FRAGMENT,
						ty: BindingType::StorageTexture {
							dimension: TextureViewDimension::D2,
							format: TextureFormat::Rgba32Float,
							readonly: true
						},
						count: None
					},
				]
			});

		let bind = device.create_bind_group(
			&BindGroupDescriptor {
				label: Some("Display/Geometry/Bind"),
				layout: &layout,
				entries: &[
					BindGroupEntry {
						binding: 0,
						resource: BindingResource::TextureView(&albedo_specular_view),
					},
					BindGroupEntry {
						binding: 1,
						resource: BindingResource::TextureView(&normal_shadow_view)
					}
				]
			});

		/* Put it all together. */
		Self {
			width,
			height,
			albedo_specular,
			normal_shadow,
			layout,
			bind
		}
	}
}

pub struct Display<A> {
	state:     A,
	swapchain: SwapChain,
	flipbook:  Consumer,

	params: RenderParameters,
	cube:   Model,
	tile:   Model,
	square: Model,

	geometry: Geometry,
	herbivores_geometry: GeometryPipeline,
	predators_geometry: GeometryPipeline,

	lighting: LightingPipeline
}
impl<A> Display<A>
	where A: Borrow<State> {

	/** Creates a new display with the given state, surface, flipbook consumer
	 * and preferences. */
	pub fn new(
		state: A,
		surface: Surface,
		flipbook: Consumer,
		prefs: &Preferences) -> Self {
		let device = state.borrow().device();

		let swapchain = device.create_swap_chain(
			&surface,
			&SwapChainDescriptor {
				usage: TextureUsage::OUTPUT_ATTACHMENT,
				format: TextureFormat::Bgra8Unorm,
				width: prefs.window.width,
				height: prefs.window.height,
				present_mode: match prefs.window.swapchain_mode {
					PresentationMode::Mailbox => PresentMode::Mailbox,
					PresentationMode::Fifo => PresentMode::Fifo
				}
			});

		/* Set up the transformations and the render parameters. */
		let world_transformation = Matrix4::scale(1.0,1.0,1.0);
		let projection = Matrix4::ortho2d(
			0.0,
			prefs.simulation.plane_width,
			0.0,
			prefs.simulation.plane_height);

		let params = RenderParameters::new(
			device,
			crate::dataset::RenderParameters {
				world_transformation,
				projection
			});

		/* Create the population pipelines. */
		let herbivores_geometry = {
			use crate::shaders::graphics::herbivore_geometry_pass as shaders;
			GeometryPipeline::new(
				device,
				&flipbook,
				Some("Display/HerbivoresGeometry/Pipeline"),
				&params,
				shaders::vertex_shader(),
				shaders::fragment_shader())
		};
		let predators_geometry = {
			use crate::shaders::graphics::predator_geometry_pass as shaders;
			GeometryPipeline::new(
				device,
				&flipbook,
				Some("Display/PredatorsGeometry/Pipeline"),
				&params,
				shaders::vertex_shader(),
				shaders::fragment_shader())
		};

		/* Create the geometry buffers. */
		let geometry = Geometry::new(
			device,
			prefs.window.width,
			prefs.window.height);

		/* Create the lighting pipeline. */
		let lighting = {
			use crate::shaders::graphics::lighting_pass as shaders;
			LightingPipeline::new(
				device,
				Some("Display/Lighting/Pipeline"),
				&geometry,
				shaders::vertex_shader(),
				shaders::fragment_shader())
		};

		/* Upload the models. */
		let cube = {
			use crate::models::basic::cube as model;
			Model::new(device, model::INDICES, model::VERTICES)
		};
		let tile = {
			use crate::models::tile as model;
			Model::new(device, model::INDICES, model::VERTICES)
		};
		let square = {
			use crate::models::basic::square as model;
			Model::new(device, model::INDICES, model::VERTICES)
		};

		Self {
			state,
			swapchain,
			flipbook,
			params,
			cube,
			tile,
			square,
			geometry,
			herbivores_geometry,
			predators_geometry,
			lighting
		}
	}

	/** Record the G-Pass onto an encoder.
	 *
	 * This pass will record normals and parameters onto the geometry buffers,
	 * that will later be used to reduce the amount of processing needed for
	 * lighting calculations down to just one screen-ful of fragments. */
	async fn record_geometry_pass(&mut self, encoder: &mut CommandEncoder) {
		/* Take a snapshot of the data buffer. */
		let snapshot = self.flipbook.snapshot();

		let albedo_specular = self.geometry
			.albedo_specular
			.create_view(
				&TextureViewDescriptor {
					label: None,
					format: Some(TextureFormat::Rgba32Float),
					dimension: Some(TextureViewDimension::D2),
					aspect: TextureAspect::All,
					base_mip_level: 0,
					level_count: None,
					base_array_layer: 0,
					array_layer_count: None
				});
		let normal_shadow = self.geometry
			.normal_shadow
			.create_view(
				&TextureViewDescriptor {
					label: None,
					format: Some(TextureFormat::Rgba32Float),
					dimension: Some(TextureViewDimension::D2),
					aspect: TextureAspect::All,
					base_mip_level: 0,
					level_count: None,
					base_array_layer: 0,
					array_layer_count: None
				});

		let mut pass = encoder.begin_render_pass(
			&RenderPassDescriptor {
				color_attachments: &[
					RenderPassColorAttachmentDescriptor {
						attachment: &albedo_specular,
						resolve_target: None,
						ops: Operations {
							load: LoadOp::Clear(Color {
								r: 0.0,
								g: 0.0,
								b: 0.0,
								a: 0.0
							}),
							store: true
						}
					},
					RenderPassColorAttachmentDescriptor {
						attachment: &normal_shadow,
						resolve_target: None,
						ops: Operations {
							load: LoadOp::Clear(Color {
								r: 0.0,
								g: 0.0,
								b: 0.0,
								a: 0.0
							}),
							store: true
						}
					}
				],
				depth_stencil_attachment: None
			});

		/* Draw the herbivores. */
		pass.set_pipeline(&self.herbivores_geometry.pipeline);
		pass.set_index_buffer(self.cube.indices());
		pass.set_vertex_buffer(0, self.cube.vertices());
		pass.set_bind_group(0, snapshot.bind_group(), &[]);
		pass.set_bind_group(1, &self.params.bind, &[]);
		pass.draw_indexed(
			0..self.cube.index_count(),
			0,
			snapshot.herbivores().await);

		/* Draw the predators. */
		pass.set_pipeline(&self.predators_geometry.pipeline);
		pass.set_index_buffer(self.cube.indices());
		pass.set_vertex_buffer(0, self.cube.vertices());
		pass.set_bind_group(0, snapshot.bind_group(), &[]);
		pass.set_bind_group(1, &self.params.bind, &[]);
		pass.draw_indexed(
			0..self.cube.index_count(),
			0,
			snapshot.predators().await);
	}

	/** Record the lighting pass onto an encoder.
	 *
	 * This pass will take the data that was recorded in the geometry pass and
	 * use it to produce lighting effects in screen space, which greatly reduces
	 * the amount of work that has to be done as the number of lights increase.
	 */
	fn record_lighting_pass(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
		let mut pass = encoder.begin_render_pass(
			&RenderPassDescriptor {
				color_attachments: &[
					RenderPassColorAttachmentDescriptor {
						attachment: view,
						resolve_target: None,
						ops: Operations {
							load: LoadOp::Clear(Color {
								r: 0.0,
								g: 0.0,
								b: 0.0,
								a: 1.0
							}),
							store: true
						}
					}
				],
				depth_stencil_attachment: None
			});

		pass.set_pipeline(&self.lighting.pipeline);
		pass.set_index_buffer(self.square.indices());
		pass.set_vertex_buffer(0, self.square.vertices());
		pass.set_bind_group(0, &self.geometry.bind, &[]);
		pass.draw_indexed(
			0..self.square.index_count,
		0,
		0..1);
	}

	pub async fn iterate(&mut self, _: Duration) {
		/* Acquire the next image in the swapchain, blocking the current thread
		 * to wait for it if it is not immediately available. */
		let image = match self.swapchain.get_current_frame() {
			Ok(image) => image,
			Err(what) => match what {
				SwapChainError::Lost | SwapChainError::Outdated => {
					/* Recreate the swapchain. */
					panic!("TODO: Recreate the swapchain here")
				},
				SwapChainError::Timeout =>
					panic!("timed out while trying to get next frame"),
				SwapChainError::OutOfMemory =>
					panic!("out of memory while trying to get the next frame")
			}
		};

		let mut encoder = self.state.borrow()
			.device()
			.create_command_encoder(
			&CommandEncoderDescriptor {
				label: Some("DrawIndividuals")
			});

		self.record_geometry_pass(&mut encoder).await;
		self.record_lighting_pass(&mut encoder, &image.output.view);

		let buffer = encoder.finish();
		self.state.borrow().queue()
			.submit(std::iter::once(buffer));
	}
}
