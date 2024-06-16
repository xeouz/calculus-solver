pub mod entity {
    use std::{any::Any, borrow::Borrow, collections::HashMap, marker::PhantomData};
    use dyn_clone::DynClone;

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum EntityKind {
        Constant,
        Variable,
        Function,
    }
    pub trait Entity: DynClone {
        fn to_str(&self) -> String;
        fn differentiate(&self) -> Box<dyn Entity>;
        fn get_kind(&self) -> EntityKind;
        fn as_any(&self) -> &dyn Any;
        fn collapse(&mut self);
    }
    
    dyn_clone::clone_trait_object!(Entity);

    pub fn try_cast_to<'a, T>(en: &Box<&'a (dyn Any + 'static)>) -> Option<&'a T> where T: Entity + 'static {
        let ret = en.downcast_ref::<T>().unwrap();

        Some(ret)
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct VariableIdentifier {
        name: String,
    }

    #[derive(Debug, Clone, Eq)]
    pub struct VariableEntity {
        variable: VariableIdentifier,
        power: i32,
    }

    impl ToString for VariableEntity {
        fn to_string(&self) -> String {
            if self.power != 1 {
                format!("{}^{}", self.variable.name, self.power)
            }
            else {
                format!("{}", self.variable.name)
            }
        }
    }
    impl PartialEq for VariableEntity {
        fn eq(&self, other: &Self) -> bool {
            self.variable.name.eq(&other.variable.name)
            &&
            self.power == other.power
        }

        fn ne(&self, other: &Self) -> bool {
            self.variable.name.ne(&other.variable.name)
            &&
            self.power != other.power
        }
    }
    impl PartialOrd for VariableEntity {
        fn ge(&self, other: &Self) -> bool {
            self.power.ge(&other.power)
        }

        fn gt(&self, other: &Self) -> bool {
            self.power.gt(&other.power)
        }

        fn le(&self, other: &Self) -> bool {
            self.power.le(&other.power)
        }

        fn lt(&self, other: &Self) -> bool {
            self.power.lt(&other.power)
        }

        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.power.partial_cmp(&other.power)
        }
    }
    impl Ord for VariableEntity {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.variable.name.cmp(&other.variable.name)
        }

        fn max(self, other: Self) -> Self
            where
                Self: Sized, {
            let mstr = self.variable.name.clone().max(other.variable.name.clone());
            if mstr == self.variable.name {
                self
            }
            else {
                other
            }
        }

        fn min(self, other: Self) -> Self
            where
                Self: Sized, {
            let mstr = self.variable.name.clone().min(other.variable.name.clone());
            if mstr == self.variable.name {
                self
            }
            else {
                other
            }
        }
        
    }

    ///--- State ---///
    pub struct DataState {
        variable_wrt: VariableIdentifier,
    }

    impl DataState {
        pub fn new(variable_wrt: VariableIdentifier) -> Self {
            Self { variable_wrt: variable_wrt }
        }
    }
    ////// State //////


    //---- Terms ----//
    #[derive(Debug, Clone)]
    pub struct ConstantTerm<'a> {
        value: f64,
        non_wrt_variables: Vec<VariableEntity>,
        kind: EntityKind,
        _phantom_data: std::marker::PhantomData<&'a ()>,
    }

    #[derive(Clone)]    
    pub struct VariableTerm<'a> {
        variable: VariableEntity,
        coeffs: Vec<Box<dyn Entity>>,
        kind: EntityKind,
        _phantom_data: std::marker::PhantomData<&'a ()>,
    }
    pub trait TermEntity {
        fn compute_result<'a>(&self) -> VariableTerm<'a>;
    }

    impl ConstantTerm<'static> {
        pub fn new(value: f64, non_wrt_variables: Vec<VariableEntity>) -> Self {
            Self { value: value, non_wrt_variables: non_wrt_variables, _phantom_data: PhantomData, kind: EntityKind::Constant }
        }

        pub fn can_add_if_collapsed(&self, other: &Self) -> bool {
            self.non_wrt_variables.eq(&other.non_wrt_variables)
        }

        pub fn add(&self, other: &Self) -> Self {
            Self::new(self.value + other.value, self.non_wrt_variables.clone())
        }
        
        pub fn multiply(&self, other: &Self) -> Self {
            let mut vars = self.non_wrt_variables.clone();
            for i in 0..vars.len() {
                vars[i].power += other.non_wrt_variables[i].power;
            }

            Self::new(self.value * other.value, vars)
        }
    }
    impl Entity for ConstantTerm<'static> {
        fn to_str(&self) -> String {
            let mut str = String::new();

            if self.value == 0.0 {
                return "0".to_string();
            }

            str += &self.value.to_string();
            str += "*";

            for var in &self.non_wrt_variables {
                str += &var.to_string();
                str += "*"
            };

            str.truncate(str.len() - 1);
            str
        }

        fn differentiate(&self) -> Box<dyn Entity> {
            Box::new(ConstantTerm::new(0.0, Vec::new()))
        }
        
        fn get_kind(&self) -> EntityKind {
            self.kind
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
        
        fn collapse(&mut self) {
            let mut vars: HashMap<String, i32> = HashMap::new();

            let list = self.non_wrt_variables.to_owned();
            for var in list.into_iter() {
                if vars.contains_key(&var.variable.name) {
                    let prev = vars.get(&var.variable.name).unwrap();
                    vars.insert(var.variable.name, prev+var.power);
                }
                else {
                    vars.insert(var.variable.name, var.power);
                }
            }

            let mut new_list:Vec<VariableEntity> = vec![];
            for (name, power) in vars.into_iter() {
                new_list.push(VariableEntity { variable: VariableIdentifier { name: name }, power: power });
            }

            new_list.sort();

            self.non_wrt_variables = new_list;
        }
    }
    impl TermEntity for ConstantTerm<'static> {
        fn compute_result<'a>(&self) -> VariableTerm<'a> {
            let variable = VariableEntity { variable: VariableIdentifier { name: "".to_string() }, power: 0 };
            
            let coeffs:Vec<Box<dyn Entity + 'static>> = vec![Box::new(ConstantTerm::new(self.value, self.non_wrt_variables.clone()))];
            VariableTerm::new(variable, coeffs)
        }
    }

    impl VariableTerm<'static> {
        pub fn new(variable: VariableEntity, coeffs: Vec<Box<dyn Entity>>) -> Self {
            Self { variable: variable, coeffs: coeffs, kind:EntityKind::Variable, _phantom_data: PhantomData }
        }
        
        pub fn equal_coeffs(&self, other: &Self) -> bool {
            for i in 0..self.coeffs.len() {
                if self.coeffs[i].get_kind() == EntityKind::Constant {
                    continue;
                }

                if self.coeffs[i].get_kind() == other.coeffs[i].get_kind() {
                    let equal = match self.coeffs[i].get_kind() {
                        EntityKind::Constant => { false },
                        EntityKind::Variable => {
                            let first = try_cast_to::<VariableTerm>(&Box::new(self.coeffs[i].as_any())).unwrap();
                            let second = try_cast_to::<VariableTerm>(&Box::new(other.coeffs[i].as_any())).unwrap();

                            first.can_add_if_collapsed(second)
                        },
                        EntityKind::Function => {
                            true
                        },
                    };

                    if !equal { return false }
                }
            };

            true
        }

        pub fn can_add_if_collapsed(&self, other: &Self) -> bool {
            self.variable == other.variable 
            &&
            self.equal_coeffs(other)
        }
        
        pub fn add(&self, other: &Self) -> Self {
            let mut coeffs = self.coeffs.clone();
            coeffs.pop();
            let constant1 = try_cast_to::<ConstantTerm>(&Box::new(self.coeffs.last().unwrap().as_any())).unwrap();
            let constant2 = try_cast_to::<ConstantTerm>(&Box::new(other.coeffs.last().unwrap().as_any())).unwrap();
            let constant_sum = constant1.add(constant2);
            coeffs.push(Box::new(constant_sum));

            Self::new(self.variable.clone(), coeffs)
        }
        
        pub fn multiply(&self, other: &Self) -> Self {
            let mut constant_product = create_number(1.0);
            let mut coeff_product: Vec<Box<dyn Entity>> = vec![];
            let mut var_product = VariableEntity { variable: VariableIdentifier { name: self.variable.variable.name.clone() }, power: 0 };

            for coeff in other.coeffs.iter() {
                match coeff.get_kind() {
                    EntityKind::Constant => {
                        let c = try_cast_to::<ConstantTerm>(&Box::new(coeff.as_any())).unwrap();
                        constant_product = Box::new(constant_product.multiply(c));
                    },
                    EntityKind::Variable => {
                        let v = try_cast_to::<VariableTerm>(&Box::new(coeff.as_any())).unwrap();
                        var_product.power += v.variable.power;
                    }
                    EntityKind::Function => {
                        coeff_product.push(coeff.clone());
                    },

                }
            };

            var_product.power += self.variable.power + other.variable.power;

            coeff_product.push(constant_product);

            Self::new(var_product, coeff_product)
        }
    }
    impl Entity for VariableTerm<'static> { 
        fn to_str(&self) -> String {
            let mut str = String::new();
            
            let mut added = false;
            for var in &self.coeffs {
                let s = var.to_str();
                if s == "1" {
                    continue;
                }
                else if s.starts_with("0") {
                    return "0".to_string();
                }
                str += &s;
                str += "*";
                added = true;
            };

            str.truncate(str.len() - (if added { 1 } else { 0 }));

            str += "";
            str += &self.variable.to_string();

            str
        }

        fn differentiate(&self) -> Box<dyn Entity> {
            let power = self.variable.power - 1;
            let variable = VariableEntity { variable: { VariableIdentifier { name: self.variable.variable.name.clone() } }, power: power};
            let mut coeffs: Vec<Box<dyn Entity>> = vec![];
            for c in self.coeffs.iter() {
                coeffs.push(dyn_clone::clone_box(&**c))
            }

            coeffs.insert(0, Box::new(ConstantTerm::new(self.variable.power.into(), vec![])));

            Box::new(VariableTerm::new(variable, coeffs))
        }
        
        fn get_kind(&self) -> EntityKind {
            self.kind
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
        
        fn collapse(&mut self) {
            let mut new_list: Vec<Box<dyn Entity>> = vec![];
            let mut constant_value: Option<ConstantTerm> = None;

            for coeff in &mut self.coeffs.iter_mut() {
                coeff.collapse();

                if coeff.get_kind() == EntityKind::Variable {
                    let v = try_cast_to::<VariableTerm>(&Box::new(coeff.as_any())).unwrap();

                    if v.variable.variable.name == self.variable.variable.name {
                        self.variable.power += v.variable.power;
                    }
                    else {
                        panic!("entity::VariableTerm::collapse(): Cannot have more than 1 variables")
                    }
                    
                    continue;
                }
                else if coeff.get_kind() == EntityKind::Constant {
                    let casted_coeff = try_cast_to::<ConstantTerm>(&Box::new(coeff.as_any()));
                    if constant_value.is_none() {
                        constant_value = casted_coeff.cloned();
                    }
                    else {
                        constant_value = Some(constant_value.unwrap().multiply(casted_coeff.unwrap()));
                    }
                    continue;
                }

                new_list.push(coeff.clone());
            }

            if constant_value.is_some() {
                let mut unwraped = constant_value.unwrap();
                unwraped.collapse();
                new_list.push(Box::new(unwraped));
            }
            else {
                new_list.push(create_number(1.0));
            }

            self.coeffs = new_list;
        }
    }
    impl TermEntity for VariableTerm<'static> {
        fn compute_result<'a>(&self) -> VariableTerm<'a> {
            self.clone()
        }
    }
    ////// Terms //////


    //---- Functions ----//
    #[derive(Clone)]
    pub struct SummationFunction {
        terms: Vec<Box<dyn Entity>>,
        kind: EntityKind,
    }
    
    #[derive(Clone)]
    pub struct MultiplicationFunction {
        first: Box<dyn Entity>,
        second: Box<dyn Entity>,
        kind: EntityKind,
    }

    pub trait Function {
        
    }


    impl SummationFunction {
        pub fn new(terms: Vec<Box<dyn Entity>>) -> Self {
            Self {
                terms: terms,
                kind: EntityKind::Function
            }
        }
    }

    impl MultiplicationFunction {
        pub fn new(first: Box<dyn Entity>, second: Box<dyn Entity>) -> Self {
            Self {
                first: first, 
                second: second,
                kind: EntityKind::Function
            }
        }
    }
    ////// Functions //////


    //---- Helper Methods ----//
    pub fn create_number(number: f64) -> Box<ConstantTerm<'static>> {
        Box::new(ConstantTerm::new(number, vec![]))
    }
    
    pub fn create_variable<'a>(name: &str, power: i32) -> Box<VariableTerm<'a>> {
        Box::new(VariableTerm::new(VariableEntity { variable: VariableIdentifier { name: name.to_string() }, power: power }, vec![]))
    }
    ////// Helper Methods //////


    //---- Differentiation ----//
    impl Entity for SummationFunction {
        fn to_str(&self) -> String {
            let mut str = String::new();

            let mut added = false;
            for var in &self.terms {
                let s = var.to_str();
                if s.starts_with("0") {
                    continue;
                }
                str += &s;
                str += " + ";
                added = true;
            };

            str.truncate(str.len() - (if added { 3 } else { 0 }));
            str
        }

        fn differentiate(&self) -> Box<dyn Entity> {
            let mut scloned = self.clone();
            scloned.collapse();

            let mut sum: Vec<Box<dyn Entity>> = vec![];
            for term in &scloned.terms {
                sum.push(term.differentiate())
            };
            scloned.terms = sum;

            scloned.collapse();
            
            Box::new(scloned)
        }
        
        fn get_kind(&self) -> EntityKind {
            self.kind
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
    
        fn collapse(&mut self) {
            for term in self.terms.iter_mut() {
                term.collapse();
            }
            
            let mut did_change = true;
            let mut change_i = 0;
            let mut change_j = 0;
            while did_change {
                did_change = false;

                let mut sum: Option<Box<dyn Entity>> = None;
                for (i, first) in self.terms.iter().enumerate() {
                    for (j, second) in self.terms.iter().enumerate() {
                        if i == j {
                            continue
                        }
    
                        //--- Check if we can add ---//
                        if first.get_kind() != second.get_kind() {
                            continue;
                        }
    
                        sum = match first.get_kind() {
                            EntityKind::Constant => {
                                let firstc = try_cast_to::<ConstantTerm>(&Box::new(first.as_any())).unwrap();
                                let secondc = try_cast_to::<ConstantTerm>(&Box::new(second.as_any())).unwrap();
    
                                if firstc.can_add_if_collapsed(secondc) {
                                    Some(Box::new(firstc.add(secondc)))
                                }
                                else {
                                    None
                                }
                            },
                            EntityKind::Variable => {
                                let firstc = try_cast_to::<VariableTerm>(&Box::new(first.as_any())).unwrap();
                                let secondc = try_cast_to::<VariableTerm>(&Box::new(second.as_any())).unwrap();
    
                                if firstc.can_add_if_collapsed(secondc) {
                                    Some(Box::new(firstc.add(secondc)))
                                }
                                else {
                                    None
                                }
                            },
                            EntityKind::Function => {
                                None
                            }
                        };

                        if sum.is_some() {
                            change_i = i;
                            change_j = j;
                            did_change = true;
                        }
                        break;
                    }
                }

                if did_change {
                    self.terms.swap_remove(change_i);
                    self.terms.swap_remove(change_j);
                    self.terms.insert(0, sum.unwrap())
                }
            }


        }
    }
    impl Function for SummationFunction {

    }

    impl Entity for MultiplicationFunction {
        fn to_str(&self) -> String {
            let mut str = String::new();

            let s1 = self.first.to_str();
            let s2 = self.second.to_str();

            if s1 == "1" {
                str += &s2;
            }
            else if s2 == "1" {
                str += &s1;
            }
            else {
                str += &self.first.to_str();
                str += "*";
                str += &self.second.to_str();
            }

            str
        }

        fn differentiate(&self) -> Box<dyn Entity> {
            let mut scloned = self.clone();
            scloned.collapse();

            let dfirst = scloned.first.differentiate();
            let dsecond = scloned.second.differentiate();

            let nfirst = Box::new(MultiplicationFunction::new(scloned.first.clone(), dsecond));
            let nsecond = Box::new(MultiplicationFunction::new(scloned.second.clone(), dfirst));
            
            let mut sum = SummationFunction::new(vec![nfirst, nsecond]);
            sum.collapse();

            Box::new(sum)
        }
        
        fn get_kind(&self) -> EntityKind {
            self.kind
        }
        
        fn as_any(&self) -> &dyn Any {
            self
        }
        
        fn collapse(&mut self) {
            self.first.collapse();
            self.second.collapse();

            if self.first.get_kind() == self.second.get_kind() {
                let product: Box<dyn Entity> = match self.first.get_kind() {
                    EntityKind::Constant => {
                        let firstc = try_cast_to::<ConstantTerm>(&Box::new(self.first.as_any())).unwrap();
                        let secondc = try_cast_to::<ConstantTerm>(&Box::new(self.second.as_any())).unwrap();

                        Box::new(firstc.multiply(secondc))
                    },
                    EntityKind::Variable => {
                        let firstc = try_cast_to::<VariableTerm>(&Box::new(self.first.as_any())).unwrap();
                        let secondc = try_cast_to::<VariableTerm>(&Box::new(self.second.as_any())).unwrap();

                        Box::new(firstc.multiply(secondc))
                    },
                    EntityKind::Function => {
                        Box::new(MultiplicationFunction::new(self.first.clone(), self.second.clone()))
                    }
                };
                
                println!("{}", product.to_str());
                self.first = product;
                self.second = create_number(1.0);
            }
        
            if self.first.to_str() == "0" || self.second.to_str() == "0" {
                self.first = create_number(0.0);
                self.second = create_number(0.0);
            }
        }
    }
    impl Function for MultiplicationFunction {

    }

    ////// Differentiation //////
}