use gdal::errors::Error as GdalError;
use serde_json::error::Error as SerdeJsonError;

error_chain! {

    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Io(::std::io::Error);
        SerdeJson(SerdeJsonError);
        GdalError(GdalError);
    }

    errors {
        UnknownDataset(name: String) {
            description("The requested dataset is unknown.")
            display("There is no dataset with name: '{}'", name)
        }
        MissingWmsParam(param: &'static str) {
            description("A mandatory WMS parameter is missing.")
            display("The following WMS parameter is missing: '{}'", param)
        }
    }

}