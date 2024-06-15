pub mod entity {
    use std::marker::PhantomData;
    use dyn_clone::DynClone;

    pub trait Entity: DynClone {
        fn to_str(&self) -> String;
        fn differentiate(&self) -> Box<dyn Entity>;
    }

    dyn_clone::clone_trait_object!(Entity);

    #[derive(Debug, Clone)]
    pub struct VariableIdentifier {
        name: String,
    }

    #[derive(Debug, Clone)]
    pub struct VariableEntity {
        variable: VariableIdentifier,
        power: i32,
    }

    impl ToString for VariableEntity {
        fn to_string(&self) -> String {
            format!("{}^{}", self.variable.name, self.power)
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
        _phantom_data: std::marker::PhantomData<&'a ()>,
    }

    #[derive(Clone)]    
    pub struct VariableTerm {
        variable: VariableEntity,
        coeffs: Vec<Box<dyn Entity>>,
    }
    pub trait TermEntity {
        fn compute_result(&self) -> VariableTerm;
    }

    impl ConstantTerm<'static> {
        pub fn new(value: f64, non_wrt_variables: Vec<VariableEntity>) -> Self {
            Self { value: value, non_wrt_variables: non_wrt_variables, _phantom_data: PhantomData }
        }
    }
    impl Entity for ConstantTerm<'static> {
        fn to_str(&self) -> String {
            let mut str = String::new();

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
    }
    impl TermEntity for ConstantTerm<'static> {
        fn compute_result(&self) -> VariableTerm {
            let variable = VariableEntity { variable: VariableIdentifier { name: "".to_string() }, power: 0 };
            
            let coeffs:Vec<Box<dyn Entity + 'static>> = vec![Box::new(ConstantTerm::new(self.value, self.non_wrt_variables.clone()))];
            VariableTerm::new(variable, coeffs)
        }
    }

    impl VariableTerm {
        pub fn new(variable: VariableEntity, coeffs: Vec<Box<dyn Entity>>) -> Self {
            Self {variable: variable, coeffs: coeffs}
        }
    }
    impl Entity for VariableTerm { 
        fn to_str(&self) -> String {
            let mut str = String::new();

            for var in &self.coeffs {
                str += &var.to_str();
                str += "*"
            };

            str.truncate(str.len() - 1);
            str += "*";
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
    }
    impl TermEntity for VariableTerm {
        fn compute_result(&self) -> VariableTerm {
            self.clone()
        }
    }
    ////// Terms //////


    //---- Functions ----//
    #[derive(Clone)]
    pub struct SummationFunction {
        terms: Vec<Box<dyn Entity>>
    }
    
    #[derive(Clone)]
    pub struct MultiplicationFunction {
        first: Box<dyn Entity>,
        second: Box<dyn Entity>,
    }

    pub trait Function {
        
    }


    impl SummationFunction {
        pub fn new(terms: Vec<Box<dyn Entity>>) -> Self {
            Self {
                terms: terms,
            }
        }
    }

    impl MultiplicationFunction {
        pub fn new(first: Box<dyn Entity>, second: Box<dyn Entity>) -> Self {
            Self {
                first: first, 
                second: second,
            }
        }
    }
    ////// Functions //////


    //---- Helper Methods ----//
    pub fn create_number(number: f64) -> Box<ConstantTerm<'static>> {
        Box::new(ConstantTerm::new(number, vec![]))
    }
    
    pub fn create_variable(name: String, power: i32) -> Box<VariableTerm> {
        Box::new(VariableTerm::new(VariableEntity { variable: VariableIdentifier { name: name }, power: power }, vec![]))
    }
    ////// Helper Methods //////


    //---- Differentiation ----//
    impl Entity for SummationFunction {
        fn to_str(&self) -> String {
            let mut str = String::new();

            for var in &self.terms {
                str += &var.to_str();
                str += " + "
            };

            str.truncate(str.len() - 3);
            str
        }

        fn differentiate(&self) -> Box<dyn Entity> {
            let mut sum: Vec<Box<dyn Entity>> = vec![];

            for term in &self.terms {
                sum.push(term.differentiate())
            };
            
            Box::new(SummationFunction::new(sum))
        }
    }
    impl Function for SummationFunction {

    }

    impl Entity for MultiplicationFunction {
        fn to_str(&self) -> String {
            let mut str = String::new();

            str += &self.first.to_str();
            str += " + ";
            str += &self.second.to_str();

            str.truncate(str.len() - 3);
            str
        }

        fn differentiate(&self) -> Box<dyn Entity> {
            let dfirst = self.first.differentiate();
            let nfirst = Box::new(MultiplicationFunction::new(self.first.clone(), dfirst));

            let dsecond = self.second.differentiate();
            let nsecond = Box::new(MultiplicationFunction::new(self.second.clone(), dsecond));

            Box::new(SummationFunction::new(vec![nfirst, nsecond]))
        }
    }

    ////// Differentiation //////
}