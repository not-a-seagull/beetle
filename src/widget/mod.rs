/* -----------------------------------------------------------------------------------
 * src/widget/mod.rs - Define the core Widget struct.
 * beetle - Simple graphics framework for Rust
 * Copyright © 2020 not_a_seagull
 *
 * This project is licensed under either the Apache 2.0 license or the MIT license, at
 * your option. For more information, please consult the LICENSE-APACHE or LICENSE-MIT
 * files in the repository root.
 * -----------------------------------------------------------------------------------
 * MIT License:
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the “Software”), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 * -----------------------------------------------------------------------------------
 * Apache 2.0 License Declaration:
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ----------------------------------------------------------------------------------
 */

mod impls;
pub mod internal;
use internal::*;
mod reference;
pub use reference::*;

use crate::object::PeerObject;
use euclid::default::Rect;
use owning_ref::{RefMutRefMut, RefRef};
use std::{cell::RefCell, fmt, sync::Arc};

/// A GUI widget.
///
/// This struct represents all GUI widgets. It acts as a wrapper around a peer object, such
/// as a Label, MainWindow, etc. It stores data related to the peer object and forwards
/// calls to it. The Widget is also implemented as an Arc<RefCell<T>>, so it can be cheaply
/// cloned and most calls to it do not require mutable access.
#[derive(Debug)]
pub struct Widget<Inner: PeerObject + 'static> {
    internal: Arc<RefCell<WidgetInternal<Inner>>>,
    generic_ref: Arc<RefCell<dyn GenericWidgetInternal>>,
}

impl<Inner: PeerObject + 'static> Clone for Widget<Inner> {
    fn clone(&self) -> Self {
        Self::from_internal(self.internal.clone())
    }
}

impl<Inner: PeerObject + 'static> Widget<Inner> {
    /// Create a new Widget from the internal Arc.
    #[inline]
    pub(crate) fn from_internal(internal: Arc<RefCell<WidgetInternal<Inner>>>) -> Self {
        Self {
            generic_ref: internal.clone(),
            internal,
        }
    }

    /// Get the internal Arc of the Widget.
    #[inline]
    pub(crate) fn internal(&self) -> &Arc<RefCell<WidgetInternal<Inner>>> {
        &self.internal
    }

    /// Create a new Widget from an inner peer object.
    #[inline]
    pub(crate) fn from_inner(inner: Inner, bounds: Rect<u32>) -> Self {
        Self::from_internal(Arc::new(RefCell::new(WidgetInternal::<Inner>::from_inner(
            inner, bounds,
        ))))
    }

    /// Get a reference to the inner peer object.
    #[inline]
    pub fn inner(&self) -> Result<RefRef<'_, WidgetInternal<Inner>, Inner>, crate::Error> {
        Ok(RefRef::new(self.internal().try_borrow()?).map(|i| i.inner()))
    }

    /// Get a mutable reference to the inner peer object.
    #[inline]
    pub fn inner_mut(
        &self,
    ) -> Result<RefMutRefMut<'_, WidgetInternal<Inner>, Inner>, crate::Error> {
        Ok(RefMutRefMut::new(self.internal().try_borrow_mut()?).map_mut(|i| i.inner_mut()))
    }
}

/// Trait that applies to all GUI widgets.
///
/// This trait solves the issue of "what if we want something to be accessible to all
/// instances of a widget?" GenericWidget is not only applied to Widget<T>, but also to
/// GenericWidgetReference, meaning that any reference to a Widget<T> can be used as a
/// widget, even if we are unsure of its type.
pub trait GenericWidget: fmt::Debug {
    /// Convert this item to a generic reference.
    fn generic_reference(&self) -> GenericWidgetReference;

    /// A generic reference to the internal Arc container.
    fn internal_generic(&self) -> Result<&Arc<RefCell<dyn GenericWidgetInternal>>, crate::Error>;

    /// The ID of this widget that uniquely identifies it.
    #[inline]
    fn id(&self) -> Result<u64, crate::Error> {
        Ok(self.internal_generic()?.try_borrow()?.id())
    }

    /// A generic reference to the inner peer object.
    #[inline]
    fn inner_generic(
        &self,
    ) -> Result<RefRef<'_, dyn GenericWidgetInternal, dyn PeerObject>, crate::Error> {
        Ok(RefRef::new(self.internal_generic()?.try_borrow()?).map(|r| r.inner_generic()))
    }

    /// A mutable generic reference to the inner peer object.
    #[inline]
    fn inner_generic_mut(
        &self,
    ) -> Result<RefMutRefMut<'_, dyn GenericWidgetInternal, dyn PeerObject>, crate::Error> {
        Ok(
            RefMutRefMut::new(self.internal_generic()?.try_borrow_mut()?)
                .map_mut(|r| r.inner_generic_mut()),
        )
    }

    /// The bounds (x/y/width/height) of this widget.
    #[inline]
    fn bounds(&self) -> Result<Rect<u32>, crate::Error> {
        Ok(self.internal_generic()?.try_borrow()?.bounds())
    }
    /// Set the bounds (x/y/width/height) of this widget.
    #[inline]
    fn set_bounds(&self, bounds: Rect<u32>) -> Result<(), crate::Error> {
        self.internal_generic()?
            .try_borrow_mut()?
            .set_bounds(bounds)
    }

    /// The parent widget for this object.
    #[inline]
    fn parent(
        &self,
    ) -> Result<RefRef<'_, dyn GenericWidgetInternal, Option<GenericWidgetReference>>, crate::Error>
    {
        Ok(RefRef::new(self.internal_generic()?.try_borrow()?).map(|r| r.parent()))
    }

    /// Set the parent widget for this object.
    ///
    /// Note: This will also add this widget as a child for the other item.
    #[inline]
    fn set_parent(&self, parent: &dyn GenericWidget) -> Result<(), crate::Error> {
        set_parent_internal(parent.generic_reference(), self.generic_reference())
    }

    /// The list of children for this object.
    #[inline]
    fn children(
        &self,
    ) -> Result<RefRef<'_, dyn GenericWidgetInternal, [GenericWidgetReference]>, crate::Error> {
        Ok(RefRef::new(self.internal_generic()?.try_borrow()?).map(|r| r.children()))
    }
    /// Add a child to this widget.
    #[inline]
    fn add_child(&self, child: &dyn GenericWidget) -> Result<(), crate::Error>
    where
        Self: Sized,
    {
        child.set_parent(self)
    }
    /// Remove a child from this widget.
    #[inline]
    fn remove_child(&self, child: &dyn GenericWidget) -> Result<(), crate::Error> {
        self.internal_generic()?
            .try_borrow_mut()?
            .remove_child(child.id()?)
    }
}

/// helper function for setting parent
pub(crate) fn set_parent_internal(
    parent: GenericWidgetReference,
    child: GenericWidgetReference,
) -> Result<(), crate::Error> {
    // remove from current parent's children list
    let imm_borrow = child.internal_generic()?.try_borrow()?;
    if let Some(current_parent) = imm_borrow.parent() {
        current_parent
            .internal_generic()?
            .try_borrow_mut()?
            .remove_child(imm_borrow.id())?;
    }

    child
        .internal_generic()?
        .try_borrow_mut()?
        .set_parent(Some(parent.clone()))?;
    parent
        .internal_generic()?
        .try_borrow_mut()?
        .add_child(&child)
}

impl<Inner: PeerObject + 'static> GenericWidget for Widget<Inner> {
    #[inline]
    fn internal_generic(&self) -> Result<&Arc<RefCell<dyn GenericWidgetInternal>>, crate::Error> {
        Ok(&self.generic_ref)
    }

    #[inline]
    fn generic_reference(&self) -> GenericWidgetReference {
        let generic: Arc<RefCell<dyn GenericWidgetInternal>> = self.internal.clone();
        GenericWidgetReference::from_reference(generic)
    }
}
