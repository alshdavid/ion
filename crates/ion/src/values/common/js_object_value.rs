#![allow(unused)]
use crate::platform::Value;

use super::*;

pub trait JsObjectValue: JsValue {
    /// Set the property value to the `Object`
    fn set_property<K, V>(
        &mut self,
        key: K,
        value: V,
    ) -> crate::Result<()>
    where
        K: JsValue,
        V: JsValue,
    {
        let env = self.env();
        let scope = &mut env.scope();

        let object_value = self.value();
        let object_raw = object_value.inner();
        let object = object_raw.cast::<v8::Object>();

        let key_value = key.value();
        let key = key_value.inner();

        let value_value = value.value();
        let value = value_value.inner();

        object.set(scope, key, value);
        Ok(())
    }

    /// Set the property value to the `Object`
    fn set_named_property<T>(
        &mut self,
        name: &str,
        value: T,
    ) -> crate::Result<()>
    where
        T: ToJsValue,
    {
        let env = self.env();
        let scope = &mut env.scope();

        let key = crate::utils::v8::v8_create_string(scope, name)?;

        let object_value = self.value();
        let object_raw = object_value.inner();
        let object = object_raw.cast::<v8::Object>();

        let value_value = T::to_js_value(env, value)?;
        let value = value_value.inner();

        object.set(scope, key.into(), value);

        Ok(())
    }

    // /// Create a named method on the `Object`
    // fn create_named_method<K>(&mut self, name: K, function: Callback) -> crate::Result<()>
    // where
    //     K: AsRef<str>,
    // {
    //     todo!();
    // }

    /// Get the property value from the `Object`
    ///
    /// Return the `InvalidArg` error if the property is not `T`
    fn get_named_property<T>(
        &self,
        name: &str,
    ) -> crate::Result<Option<T>>
    where
        T: FromJsValue,
    {
        let env = self.env();
        let scope = &mut env.scope();

        let key = crate::utils::v8::v8_create_string(scope, name)?;

        let object_value = self.value();
        let object_raw = object_value.inner();
        let object = object_raw.cast::<v8::Object>();

        let Some(result) = object.get(scope, key.into()) else {
            return Ok(None);
        };

        Ok(Some(T::from_js_value(env, Value::from(result))?))
    }

    /// Get the property value from the `Object` without validation
    fn get_named_property_unchecked<T>(
        &self,
        name: &str,
    ) -> crate::Result<T>
    where
        T: FromJsValue,
    {
        let env = self.env();
        let scope = &mut env.scope();

        let key = crate::utils::v8::v8_create_string(scope, name)?;

        let object_value = self.value();
        let object_raw = object_value.inner();
        let object = object_raw.cast::<v8::Object>();

        let Some(result) = object.get(scope, key.into()) else {
            return Err(crate::Error::ValueGetError);
        };

        T::from_js_value(env, Value::from(result))
    }

    /// Check if the `Object` has the named property
    fn has_named_property<N: AsRef<str>>(
        &self,
        name: N,
    ) -> crate::Result<bool> {
        todo!();
    }

    /// Delete the property from the `Object`, the property name can be a `JsValue`
    fn delete_property<S>(
        &mut self,
        name: S,
    ) -> crate::Result<bool>
    where
        S: JsValue,
    {
        let env = self.env();
        let scope = &mut env.scope();

        let object_value = self.value();
        let object_raw = object_value.inner();
        let object = object_raw.cast::<v8::Object>();

        object.delete(scope, name.value().inner());
        Ok(true)
    }

    /// Delete the property from the `Object`
    fn delete_named_property<K: AsRef<str>>(
        &mut self,
        name: K,
    ) -> crate::Result<bool> {
        let env = self.env();
        let scope = &mut env.scope();

        let key = crate::utils::v8::v8_create_string(scope, name)?;

        let object_value = self.value();
        let object_raw = object_value.inner();
        let object = object_raw.cast::<v8::Object>();

        object.delete(scope, key.into());
        Ok(true)
    }

    /// Check if the `Object` has the own property
    fn has_own_property(
        &self,
        key: &str,
    ) -> crate::Result<bool> {
        todo!();
    }

    /// The same as `has_own_property`, but accepts a `JsValue` as the property name.
    fn has_own_property_js<K>(
        &self,
        key: K,
    ) -> crate::Result<bool>
    where
        K: JsValue,
    {
        todo!();
    }

    /// This API checks if the Object passed in has the named property.
    fn has_property(
        &self,
        name: &str,
    ) -> crate::Result<bool> {
        todo!();
    }

    /// This API is the same as `has_property`, but accepts a `JsValue` as the property name.
    /// So you can pass the `JsNumber` or `JsSymbol` as the property name.
    fn has_property_js<K>(
        &self,
        name: K,
    ) -> crate::Result<bool>
    where
        K: JsValue,
    {
        todo!();
    }

    // /// This API returns the names of the enumerable properties of object as an array of strings.
    // // The properties of object whose key is a symbol will not be included.
    // fn get_property_names(&self) -> crate::Result<Object<'env>> {
    //     todo!();
    // }

    // /// <https://nodejs.org/api/n-api.html#n_api_napi_get_all_property_names>
    // /// This API returns an array containing the names of the available properties of this object.
    // fn get_all_property_names(
    //     &self,
    //     mode: KeyCollectionMode,
    //     filter: KeyFilter,
    //     conversion: KeyConversion,
    // ) -> crate::Result<Object<'env>> {
    //     todo!();
    // }

    // /// This returns the equivalent of `Object.getPrototypeOf` (which is not the same as the function's prototype property).
    // fn get_prototype(&self) -> crate::Result<Unknown<'env>> {
    //     todo!();
    // }

    /// Get the prototype of the `Object`
    fn get_prototype_unchecked<T>(&self) -> crate::Result<T>
    where
        T: FromJsValue,
    {
        todo!();
    }

    /// Set the element at the given index
    fn set_element<T>(
        &mut self,
        index: u32,
        value: T,
    ) -> crate::Result<()>
    where
        T: JsValue,
    {
        todo!();
    }

    /// Check if the `Array` has the element at the given index
    fn has_element(
        &self,
        index: u32,
    ) -> crate::Result<bool> {
        todo!();
    }

    /// Delete the element at the given index
    fn delete_element(
        &mut self,
        index: u32,
    ) -> crate::Result<bool> {
        todo!();
    }

    /// Get the element at the given index
    ///
    /// If the `Object` is not an array, `ArrayExpected` error returned
    fn get_element<T>(
        &self,
        index: u32,
    ) -> crate::Result<T>
    where
        T: FromJsValue,
    {
        todo!();
    }

    // /// This method allows the efficient definition of multiple properties on a given object.
    // fn define_properties(&mut self, properties: &[Property]) -> crate::Result<()> {
    //     todo!();
    // }

    /// Perform `is_array` check before get the length
    ///
    /// if `Object` is not array, `ArrayExpected` error returned
    fn get_array_length(&self) -> crate::Result<u32> {
        todo!();
    }

    /// use this API if you can ensure this `Object` is `Array`
    fn get_array_length_unchecked(&self) -> crate::Result<u32> {
        todo!();
    }

    /// Wrap the native value `T` to this `Object`
    /// the `T` will be dropped when this `Object` is finalized
    fn wrap<T: 'static>(
        &mut self,
        native_object: T,
        size_hint: Option<usize>,
    ) -> crate::Result<()> {
        todo!();
    }

    /// Get the wrapped native value from the `Object`
    ///
    /// Return the `InvalidArg` error if the `Object` is not wrapped the `T`
    fn unwrap<T: 'static>(&self) -> crate::Result<&mut T> {
        todo!();
    }

    /// Remove the wrapped native value from the `Object`
    ///
    /// Return the `InvalidArg` error if the `Object` is not wrapped the `T`
    fn remove_wrapped<T: 'static>(&mut self) -> crate::Result<()> {
        todo!();
    }

    // /// Adds a `finalize_cb` callback which will be called when the JavaScript object in js_object has been garbage-collected.
    // ///
    // /// This API can be called multiple times on a single JavaScript object.
    // fn add_finalizer<T, Hint, F>(
    //     &mut self,
    //     native: T,
    //     finalize_hint: Hint,
    //     finalize_cb: F,
    // ) -> crate::Result<()>
    // where
    //     T: 'static,
    //     Hint: 'static,
    //     F: FnOnce(FinalizeContext<T, Hint>) + 'static,
    // {
    //     todo!();
    // }

    /// This method freezes a given object.
    /// This prevents new properties from being added to it, existing properties from being removed, prevents changing the enumerability, configurability, or writability of existing properties, and prevents the values of existing properties from being changed.
    /// It also prevents the object's prototype from being changed. This is described in [Section 19.1.2.6](https://tc39.es/ecma262/#sec-object.freeze) of the ECMA-262 specification.
    fn freeze(&mut self) -> crate::Result<()> {
        todo!();
    }

    /// This method seals a given object. This prevents new properties from being added to it, as well as marking all existing properties as non-configurable.
    /// This is described in [Section 19.1.2.20](https://tc39.es/ecma262/#sec-object.seal) of the ECMA-262 specification.
    fn seal(&mut self) -> crate::Result<()> {
        todo!();
    }
}
