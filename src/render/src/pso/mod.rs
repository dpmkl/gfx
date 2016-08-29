// Copyright 2015 The Gfx-rs Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A typed high-level graphics pipeline interface.
//!
//! # Overview
//! A `PipelineState` holds all information needed to manage a graphics pipeline. It contains
//! information about the shaders used, and on how to bind variables to these shaders. A 
//! `PipelineState` manifests itself in the form of a Pipeline State Object, or PSO in short.
//!
//! A Pipeline State Object exists out of different components. Every component represents
//! a resource handle: a shader input or output/target. The types of these components can be found
//! in this module's submodules, grouped by category.
//!
//! Before all, a Pipeline State Object must be defined. This is done using the `gfx_pipeline`
//! macro. This macro creates three different structures:
//!
//! - The `Init` structure contains the location of every PSO component. During shader linking,
//!   this is used to construct the `Meta` structure. 
//! - The `Meta` structure contains the layout of every PSO. Using the `Meta` structure, the right
//!   data is mapped to the right components.
//! - The `Data` structure contains the data of all components, to be sent to the GPU. 
//!
//! # Construction and Handling
//! A Pipeline State Object is constructed by a factory, from its `Init` structure, a `Rasterizer`,
//! a primitive type and a shader program.
//!
//! After construction an `Encoder` can use the PSO along with a `Data` structure matching that
//! PSO to process the shader pipeline, for instance, using the `draw` method.

pub mod buffer;
pub mod resource;
pub mod target;
pub mod bundle;

use std::default::Default;
use std::error::Error;
use std::fmt;
use gfx_core as d;
pub use gfx_core::pso::{Descriptor};

/// Informations about what is accessed by the pipeline
pub type AccessInfo<R> = ::gfx_core::pso::AccessInfo<R>;

/// A complete set of raw data that needs to be specified at run-time
/// whenever we draw something with a PSO. This is what "data" struct
/// gets transformed into when we call `encoder.draw(...)` with it.
/// It doesn't have any typing information, since PSO knows what
/// format and layout to expect from each resource.
#[allow(missing_docs)]
#[derive(Debug)]
pub struct RawDataSet<R: d::Resources>{
    pub vertex_buffers: d::pso::VertexBufferSet<R>,
    pub constant_buffers: Vec<d::pso::ConstantBufferParam<R>>,
    pub global_constants: Vec<(d::shade::Location, d::shade::UniformValue)>,
    pub resource_views: Vec<d::pso::ResourceViewParam<R>>,
    pub unordered_views: Vec<d::pso::UnorderedViewParam<R>>,
    pub samplers: Vec<d::pso::SamplerParam<R>>,
    pub pixel_targets: d::pso::PixelTargetSet<R>,
    pub ref_values: d::state::RefValues,
    pub scissor: d::target::Rect,
}

impl<R: d::Resources> RawDataSet<R> {
    /// Create an empty data set.
    pub fn new() -> RawDataSet<R> {
        RawDataSet {
            vertex_buffers: d::pso::VertexBufferSet::new(),
            constant_buffers: Vec::new(),
            global_constants: Vec::new(),
            resource_views: Vec::new(),
            unordered_views: Vec::new(),
            samplers: Vec::new(),
            pixel_targets: d::pso::PixelTargetSet::new(),
            ref_values: Default::default(),
            scissor: d::target::Rect{x:0, y:0, w:1, h:1},
        }
    }
    /// Clear all contained data.
    pub fn clear(&mut self) {
        self.vertex_buffers = d::pso::VertexBufferSet::new();
        self.constant_buffers.clear();
        self.global_constants.clear();
        self.resource_views.clear();
        self.unordered_views.clear();
        self.samplers.clear();
        self.pixel_targets = d::pso::PixelTargetSet::new();
        self.ref_values = Default::default();
        self.scissor = d::target::Rect{x:0, y:0, w:1, h:1};
    }
}

/// Error matching an element inside the constant buffer.
#[derive(Clone, Debug, PartialEq)]
pub enum ElementError<S> {
    /// Element not found.
    NotFound(S),
    /// Element offset mismatch.
    Offset(S, d::pso::ElemOffset),
    /// Element format mismatch.
    Format(S, d::shade::ConstFormat),
}

impl<'a> From<ElementError<&'a str>> for ElementError<String> {
    fn from(other: ElementError<&'a str>) -> ElementError<String> {
        use self::ElementError::*;
        match other {
            NotFound(s) => NotFound(s.to_owned()),
            Offset(s, v) => Offset(s.to_owned(), v),
            Format(s, v) => Format(s.to_owned(), v),
        }
    }
}


/// Failure to initilize the link between the shader and the data.
#[derive(Clone, PartialEq, Debug)]
pub enum InitError<S> {
    /// Vertex attribute mismatch.
    VertexImport(S, Option<d::format::Format>),
    /// Constant buffer mismatch.
    ConstantBuffer(S, Option<ElementError<S>>),
    /// Global constant mismatch.
    GlobalConstant(S, Option<()>),
    /// Shader resource view mismatch.
    ResourceView(S, Option<()>),
    /// Unordered access view mismatch.
    UnorderedView(S, Option<()>),
    /// Sampler mismatch.
    Sampler(S, Option<()>),
    /// Pixel target mismatch.
    PixelExport(S, Option<d::format::Format>),
}

impl<'a> From<InitError<&'a str>> for InitError<String> {
    fn from(other: InitError<&'a str>) -> InitError<String> {
        use self::InitError::*;
        match other {
            VertexImport(s, v) => VertexImport(s.to_owned(), v),
            ConstantBuffer(s, v) => ConstantBuffer(s.to_owned(), v.map(|e| e.into())),
            GlobalConstant(s, v) => GlobalConstant(s.to_owned(), v),
            ResourceView(s, v) => ResourceView(s.to_owned(), v),
            UnorderedView(s, v) => UnorderedView(s.to_owned(), v),
            Sampler(s, v) => Sampler(s.to_owned(), v),
            PixelExport(s, v) => PixelExport(s.to_owned(), v),
        }
    }
}

impl<S: Error> fmt::Display for InitError<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::InitError::*;
        let desc = self.description();
        match *self {
            VertexImport(ref name, format) => write!(f, "{}: ({}, {:?})", desc, name, format),
            ConstantBuffer(ref name, ref opt) => write!(f, "{}: ({}, {:?})", desc, name, opt),
            GlobalConstant(ref name, opt) => write!(f, "{}: ({}, {:?})", desc, name, opt),
            ResourceView(ref name, opt) => write!(f, "{}: ({}, {:?})", desc, name, opt),
            UnorderedView(ref name, opt) => write!(f, "{}: ({}, {:?})", desc, name, opt),
            Sampler(ref name, opt) => write!(f, "{}: ({}, {:?})", desc, name, opt),
            PixelExport(ref name, format) => write!(f, "{}: ({}, {:?})", desc, name, format),
        }
    }
}

impl<S: Error> Error for InitError<S> {
    fn description(&self) -> &str {
        use self::InitError::*;
        match *self {
            VertexImport(_, None) => "Vertex attribute not found",
            VertexImport(..) => "Vertex attribute format mismatch",
            ConstantBuffer(_, None) => "Constant buffer not found",
            ConstantBuffer(..) => "Constant buffer element mismatch",
            GlobalConstant(_, None) => "Global constant not found",
            GlobalConstant(..) => "Global constant format mismatch",
            ResourceView(_, None) => "Shader resource view not found",
            ResourceView(..) => "Shader resource view mismatch",
            UnorderedView(_, None) => "Unordered access view not found",
            UnorderedView(..) => "Unordered access view mismatch",
            Sampler(_, None) => "Sampler not found",
            Sampler(..) => "Sampler mismatch",
            PixelExport(_, None) => "Pixel target not found",
            PixelExport(..) => "Pixel target mismatch",
        }
    }
}


/// A service trait implemented by the "init" structure of PSO.
pub trait PipelineInit {
    /// The associated "meta" struct.
    type Meta;
    /// Attempt to map a PSO descriptor to a give shader program,
    /// represented by `ProgramInfo`. Returns an instance of the
    /// "meta" struct upon successful mapping.
    fn link_to<'s>(&self, &mut Descriptor, &'s d::shade::ProgramInfo)
               -> Result<Self::Meta, InitError<&'s str>>;
}

/// a service trait implemented the "data" structure of PSO.
pub trait PipelineData<R: d::Resources> {
    /// The associated "meta" struct.
    type Meta;
    /// Dump all the contained data into the raw data set,
    /// given the mapping ("meta"), and a handle manager.
    fn bake_to(&self,
               &mut RawDataSet<R>,
               &Self::Meta,
               &mut d::handle::Manager<R>,
               &mut AccessInfo<R>);
}

/// A strongly typed Pipleline State Object. See the module documentation for more information.
pub struct PipelineState<R: d::Resources, M>(
    d::handle::RawPipelineState<R>, d::Primitive, M);

impl<R: d::Resources, M> PipelineState<R, M> {
    /// Create a new PSO from a raw handle and the "meta" instance.
    pub fn new(raw: d::handle::RawPipelineState<R>, prim: d::Primitive, meta: M)
               -> PipelineState<R, M> {
        PipelineState(raw, prim, meta)
    }
    /// Get a raw handle reference.
    pub fn get_handle(&self) -> &d::handle::RawPipelineState<R> {
        &self.0
    }
    /// Get a "meta" struct reference. Can be used by the user to check
    /// what resources are actually used and what not.
    pub fn get_meta(&self) -> &M {
        &self.2
    }
}

/// The "link" logic portion of a PSO component.
/// Defines the input data for the component.
pub trait DataLink<'a>: Sized {
    /// The assotiated "init" type - a member of the PSO "init" struct.
    type Init: 'a;
    /// Create a new empty data link.
    fn new() -> Self;
    /// Check if this link is actually used by the shader.
    fn is_active(&self) -> bool;
    /// Attempt to link with a vertex buffer containing multiple attributes.
    fn link_vertex_buffer(&mut self, _: d::pso::BufferIndex, _: &Self::Init) ->
                          Option<d::pso::VertexBufferDesc> { None }
    /// Attempt to link with a vertex attribute.
    fn link_input(&mut self, _: &d::shade::AttributeVar, _: &Self::Init) ->
                  Option<Result<d::pso::AttributeDesc, d::format::Format>> { None }
    /// Attempt to link with a constant buffer.
    fn link_constant_buffer<'b>(&mut self, _: &'b d::shade::ConstantBufferVar, _: &Self::Init) ->
                            Option<Result<d::pso::ConstantBufferDesc, ElementError<&'b str>>> { None }
    /// Attempt to link with a global constant.
    fn link_global_constant(&mut self, _: &d::shade::ConstVar, _: &Self::Init) ->
                            Option<Result<(), d::shade::UniformValue>> { None }
    /// Attempt to link with an output render target (RTV).
    fn link_output(&mut self, _: &d::shade::OutputVar, _: &Self::Init) ->
                   Option<Result<d::pso::ColorTargetDesc, d::format::Format>> { None }
    /// Attempt to link with a depth-stencil target (DSV).
    fn link_depth_stencil(&mut self, _: &Self::Init) ->
                          Option<d::pso::DepthStencilDesc> { None }
    /// Attempt to link with a shader resource (SRV).
    fn link_resource_view(&mut self, _: &d::shade::TextureVar, _: &Self::Init) ->
                          Option<Result<d::pso::ResourceViewDesc, d::format::Format>> { None }
    /// Attempt to link with an unordered access (UAV).
    fn link_unordered_view(&mut self, _: &d::shade::UnorderedVar, _: &Self::Init) ->
                           Option<Result<d::pso::UnorderedViewDesc, d::format::Format>> { None }
    /// Attempt to link with a sampler.
    fn link_sampler(&mut self, _: &d::shade::SamplerVar, _: &Self::Init)
                    -> Option<d::pso::SamplerDesc> { None }
    /// Attempt to enable scissor test.
    fn link_scissor(&mut self) -> bool { false }
}

/// The "bind" logic portion of the PSO component.
/// Defines how the user data translates into the raw data set.
pub trait DataBind<R: d::Resources> {
    /// The associated "data" type - a member of the PSO "data" struct.
    type Data;
    /// Dump the given data into the raw data set.
    fn bind_to(&self,
               &mut RawDataSet<R>,
               &Self::Data,
               &mut d::handle::Manager<R>,
               &mut AccessInfo<R>);
}