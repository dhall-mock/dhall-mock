let HttpMethod = < CONNECT | DELETE | GET | HEAD | OPTIONS | PATCH | POST | PUT | TRACE >

let Body = < JSON : { json : Text } | TEXT : { text : Text } >

let QueryParam = { key: Text, value: Text }

let Header = { mapKey: Text, mapValue: Text }

let HttpRequest
    = { Type = { method  : Optional HttpMethod
               , path    : Optional Text
               , body    : Optional Body
               , params  : List QueryParam
               , headers : List Header
               }
      , default = { method  = None HttpMethod
                  , path    = None Text
                  , body    = None Body
                  , params  = [] : List QueryParam
                  , headers = [] : List Header
                  }
      }

let HttpResponse 
    = { Type = { statusCode   : Optional Natural
               , statusReason : Optional Text
               , body         : Optional Text
               , headers      : List Header
               }
      , default = { statusCode   = None Natural
                  , statusReason = None Text
                  , body         = None Text
                  , headers      = [] : List Header
                  }
      }

let Expectation : Type =
      { request  : HttpRequest.Type
      , response : HttpResponse.Type
      }

let contentTypeJSON : Header = 
  { mapKey = "Content-Type", mapValue = "application/json" }

let contentTypeXML : Header = 
  { mapKey = "Content-Type", mapValue = "application/xml" }

let contentTypeText : Header = 
  { mapKey = "Content-Type", mapValue = "text/plain"}

in { HttpMethod      = HttpMethod
   , QueryParam      = QueryParam
   , Header          = Header
   , Body            = Body
   , HttpRequest     = HttpRequest
   , HttpResponse    = HttpResponse
   , Expectation     = Expectation
   , statusOK            = Some 200
   , statusCreated       = Some 201
   , statusBadRequest    = Some 400
   , statusUnauthorized  = Some 401
   , statusForbidden     = Some 403
   , statusNotFound      = Some 404
   , statusInternalError = Some 500
   , contentTypeJSON     = contentTypeJSON
   , contentTypeXML      = contentTypeXML
   }
