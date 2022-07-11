//! Error map parsing. Largely based off of Diem code.

// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use docstring::normalize_doc_string;
pub use errmap::*;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::Identifier, language_storage::ModuleId,
};
use move_model::{
    ast::Value,
    model::{GlobalEnv, ModuleEnv, NamedConstantEnv},
    symbol::Symbol,
};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrmapOptions {
    /// The constant prefix that determines if a constant is an error or not
    pub error_prefix: String,
    /// The module ID of the error category module
    pub error_category_module: ModuleId,
}

impl Default for ErrmapOptions {
    fn default() -> Self {
        Self {
            error_prefix: "E".to_string(),
            error_category_module: ModuleId::new(
                static_address::static_address!("0x1"),
                ident_str!("Errors").to_owned(),
            ),
        }
    }
}

pub struct ErrmapGen<'env> {
    /// Options for error map generation
    options: &'env ErrmapOptions,
    /// Input definitions
    env: &'env GlobalEnv,
    /// Output error mapping
    output: ErrorMapping,
}

trait ErrorMappingMut {
    fn add_error_category(&mut self, category_id: u64, description: ErrorDescription)
        -> Result<()>;

    fn add_module_error(
        &mut self,
        module_id: ModuleId,
        abort_code: u64,
        description: ErrorDescription,
    ) -> Result<()>;
}

impl ErrorMappingMut for ErrorMapping {
    fn add_error_category(
        &mut self,
        category_id: u64,
        description: ErrorDescription,
    ) -> Result<()> {
        if let Some(previous_entry) = self.error_categories.insert(category_id, description) {
            bail!(format!(
                "Entry for category {} already taken by: {:#?}",
                category_id, previous_entry
            ))
        }
        Ok(())
    }

    fn add_module_error(
        &mut self,
        module_id: ModuleId,
        abort_code: u64,
        description: ErrorDescription,
    ) -> Result<()> {
        let module_error_map = self
            .module_error_maps
            .entry(module_id.clone().into())
            .or_default();
        if let Some(previous_entry) = module_error_map.insert(abort_code, description) {
            bail!(format!(
                "Duplicate entry for abort code {} found in {}, previous entry: {:#?}",
                abort_code, module_id, previous_entry
            ))
        }
        Ok(())
    }
}

impl<'env> ErrmapGen<'env> {
    pub fn new(env: &'env GlobalEnv, options: &'env ErrmapOptions) -> Self {
        Self {
            options,
            env,
            output: ErrorMapping::default(),
        }
    }

    pub fn gen(&mut self) -> Result<ErrorMapping> {
        for module in self.env.get_modules() {
            if !module.is_script_module() {
                self.build_error_map(&module)?;
            }
        }
        Ok(self.output.clone())
    }

    pub fn gen_for_module(&mut self, module: &ModuleEnv) -> Result<ErrorMapping> {
        if module.is_script_module() {
            bail!("cannot generate error mappings for script modules");
        }
        self.build_error_map(module)?;
        Ok(self.output.clone())
    }

    fn build_error_map(&mut self, module: &ModuleEnv<'_>) -> Result<()> {
        let module_id = self.get_module_id_for_name(module);
        if module_id == self.options.error_category_module {
            self.build_error_categories(module)?
        } else {
            self.build_error_map_for_module(&module_id, module)?
        }
        Ok(())
    }

    fn build_error_categories(&mut self, module: &ModuleEnv<'_>) -> Result<()> {
        for named_constant in module.get_named_constants() {
            let name = self.name_string(named_constant.get_name());
            let error_category = self.get_abort_code(&named_constant)?;
            self.output.add_error_category(
                error_category,
                ErrorDescription {
                    code_name: name.to_string(),
                    code_description: normalize_doc_string(named_constant.get_doc())
                        .unwrap_or_default(),
                },
            )?
        }
        Ok(())
    }

    pub fn build_error_map_for_module(
        &mut self,
        module_id: &ModuleId,
        module: &ModuleEnv<'_>,
    ) -> Result<()> {
        for named_constant in module.get_named_constants() {
            let name = self.name_string(named_constant.get_name());
            if name.starts_with(&self.options.error_prefix) {
                let abort_code = self.get_abort_code(&named_constant)?;
                self.output.add_module_error(
                    module_id.clone(),
                    abort_code,
                    ErrorDescription {
                        code_name: name.to_string(),
                        code_description: normalize_doc_string(named_constant.get_doc())
                            .unwrap_or_default(),
                    },
                )?
            }
        }
        Ok(())
    }

    fn get_abort_code(&self, constant: &NamedConstantEnv<'_>) -> Result<u64> {
        match constant.get_value() {
            Value::Number(big_int) => u64::try_from(big_int).map_err(|err| err.into()),
            x => bail!(
                "Invalid abort code constant {} found for code {}",
                x,
                self.name_string(constant.get_name())
            ),
        }
    }

    fn get_module_id_for_name(&self, module: &ModuleEnv<'_>) -> ModuleId {
        let name = module.get_name();
        let addr = AccountAddress::from_hex_literal(&format!("0x{:x}", name.addr())).unwrap();
        let name = Identifier::new(self.name_string(name.name()).to_string()).unwrap();
        ModuleId::new(addr, name)
    }

    fn name_string(&self, symbol: Symbol) -> Rc<String> {
        self.env.symbol_pool().string(symbol)
    }
}
