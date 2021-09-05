//! product type

use super::super::rand::R;
use super::base::Generator;

macro_rules! generate_tuple {
    ($name:ident, $fct_name:ident, $(($type_name:ident, $type_param:ident),)*) => {
        #[doc = concat!(stringify!($name), " generator type , figuratively generate item of the form ",
          stringify!( ($($type_param ),*) ))]
        #[derive(Clone)]
        pub struct $name<$($type_param),*> {
            $($type_name : $type_param),*
        }

        impl<$($type_param),*> Generator for $name<$($type_param,)*>
            where $($type_param: Generator,)*
        {
            type Item = ( $($type_param::Item),* );

            fn gen(&self, r: &mut R) -> Self::Item {
                ($( self.$type_name.gen(&mut r.sub()) ),*)
            }
        }

        #[doc = concat!(stringify!($name), " generator, figuratively a tuple generator of ", stringify!( ($($type_param, )*) ))]
        pub fn $fct_name <$($type_param,)*>( $( $type_name : $type_param ,)* ) -> $name<$($type_param,)*> {
            $name { $( $type_name : $type_name,)* }
        }
    };
}

generate_tuple! {Tuple2, tuple2, (a, A), (b, B), }
generate_tuple! {Tuple3, tuple3, (a, A), (b, B), (c, C),}
generate_tuple! {Tuple4, tuple4, (a, A), (b, B), (c, C), (d, D),}
generate_tuple! {Tuple5, tuple5, (a, A), (b, B), (c, C), (d, D), (e, E),}
generate_tuple! {Tuple6, tuple6, (a, A), (b, B), (c, C), (d, D), (e, E), (f, F),}

macro_rules! generate_product {
    ($name:ident, $fct_name:ident, $(($type_name:ident, $type_param:ident),)*) => {
        #[doc = concat!(stringify!($name), " generator type , figuratively generate item of the form M",
          stringify!( ($($type_param, )*) ))]
        #[doc = ""]
        #[doc = "this is similar to the tuple generator with a added mapping function from the tuple to a given output type"]
        #[derive(Clone)]
        pub struct $name<$($type_param),* , M> {
            $($type_name : $type_param),*,
            mapper: M,
        }

        impl<$($type_param),*, M, O> Generator for $name<$($type_param),*, M>
            where
                $($type_param: Generator,)*
                M: Fn($($type_param::Item),*) -> O + Clone,
        {
            type Item = O;

            fn gen(&self, r: &mut R) -> Self::Item {
                (self.mapper)($( self.$type_name.gen(&mut r.sub()) , )*)
            }
        }

        #[doc = concat!(stringify!($name), " generator, figuratively a product generator of M", stringify!( ($($type_param, )*) ))]
        #[doc = ""]
        #[doc = "this is similar to the tuple generator with a added mapping function from the tuple to a given output type"]
        pub fn $fct_name <$($type_param),*, M>( $( $type_name : $type_param),*, mapper: M) -> $name<$($type_param),*, M> {
            $name { $( $type_name : $type_name),*, mapper }
        }
    };
}

generate_product! {Product2, product2, (a, A), (b, B), }
generate_product! {Product3, product3, (a, A), (b, B), (c, C),}
generate_product! {Product4, product4, (a, A), (b, B), (c, C), (d, D),}
generate_product! {Product5, product5, (a, A), (b, B), (c, C), (d, D), (e, E),}
generate_product! {Product6, product6, (a, A), (b, B), (c, C), (d, D), (e, E), (f, F),}
