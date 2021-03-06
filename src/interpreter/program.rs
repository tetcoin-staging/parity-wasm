use std::rc::Rc;
use std::collections::HashMap;
use std::borrow::Cow;
use elements::Module;
use interpreter::Error;
use interpreter::module::ModuleInstance;
use interpreter::func::FuncInstance;
use interpreter::host::HostModule;
use interpreter::value::RuntimeValue;
use interpreter::imports::{Imports, ImportResolver};
use interpreter::state::HostState;

/// Program instance. Program is a set of instantiated modules.
#[deprecated]
pub struct ProgramInstance {
	modules: HashMap<String, Rc<ModuleInstance>>,
	resolvers: HashMap<String, Box<ImportResolver>>,
}

impl ProgramInstance {
	/// Create new program instance.
	pub fn new() -> ProgramInstance {
		ProgramInstance {
			modules: HashMap::new(),
			resolvers: HashMap::new(),
		}
	}

	/// Instantiate module with validation.
	pub fn add_module<'a>(
		&mut self,
		name: &str,
		module: Module,
		state: &'a mut HostState<'a>,
	) -> Result<Rc<ModuleInstance>, Error> {
		let module_instance = {
			let mut imports = Imports::new();
			for (module_name, module_instance) in self.modules.iter() {
				imports.push_resolver(&**module_name, &**module_instance);
			}
			for (module_name, import_resolver) in self.resolvers.iter() {
				imports.push_resolver(&**module_name, &**import_resolver);
			}
			ModuleInstance::new(&module)
				.with_imports(imports)
				.run_start(state)?
		};
		self.modules.insert(name.to_owned(), Rc::clone(&module_instance));

		Ok(module_instance)
	}

	pub fn add_import_resolver(
		&mut self,
		name: &str,
		import_resolver: Box<ImportResolver>,
	) {
		self.resolvers.insert(name.to_owned(), import_resolver);
	}

	pub fn add_host_module(
		&mut self,
		name: &str,
		host_module: HostModule,
	) {
		self.resolvers.insert(name.to_owned(), Box::new(host_module) as Box<ImportResolver>);
	}

	pub fn insert_loaded_module(&mut self, name: &str, module: Rc<ModuleInstance>) {
		self.modules.insert(name.to_owned(), module);
	}

	pub fn invoke_export<'a>(
		&mut self,
		module_name: &str,
		func_name: &str,
		args: &[RuntimeValue],
		state: &'a mut HostState<'a>,
	) -> Result<Option<RuntimeValue>, Error> {
		let module_instance = self.modules.get(module_name).ok_or_else(|| {
			Error::Program(format!("Module {} not found", module_name))
		})?;
		module_instance.invoke_export(func_name, args, state)
	}

	pub fn invoke_index<'a>(
		&mut self,
		module_name: &str,
		func_idx: u32,
		args: &[RuntimeValue],
		state: &'a mut HostState<'a>,
	) -> Result<Option<RuntimeValue>, Error> {
		let module_instance = self.modules.get(module_name).cloned().ok_or_else(|| {
			Error::Program(format!("Module {} not found", module_name))
		})?;
		module_instance.invoke_index(func_idx, args, state)
	}

	pub fn invoke_func<'a>(
		&mut self,
		func_instance: Rc<FuncInstance>,
		args: &[RuntimeValue],
		state: &'a mut HostState<'a>,
	) -> Result<Option<RuntimeValue>, Error> {
		FuncInstance::invoke(Rc::clone(&func_instance), Cow::Borrowed(args), state)
	}

	pub fn resolver(&self, name: &str) -> Option<&ImportResolver> {
		self.modules
			.get(name)
			.map(|x| &**x as &ImportResolver)
			.or_else(|| self.resolvers.get(name).map(|x| &**x))
	}

	pub fn module(&self, name: &str) -> Option<Rc<ModuleInstance>> {
		self.modules.get(name).cloned()
	}
}
