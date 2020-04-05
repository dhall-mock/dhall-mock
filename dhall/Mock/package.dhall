let HttpMethod = < HEAD | PUT | POST | GET >

let HttpRequest : Type
    = { method  : Optional HttpMethod
      , path    : Optional Text
      }

let HttpResponse : Type =
      { statusCode   : Optional Integer
      , statusReason : Optional Text
      , body         : Optional Text
      }

let Expectation : Type =
      { request  : HttpRequest
      , response : HttpResponse
      }

in { HttpMethod   = HttpMethod
   , HttpRequest  = HttpRequest
   , HttpResponse = HttpResponse
   , Expectation  = Expectation
   }
