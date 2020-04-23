let Mock = ./dhall/Mock/package.dhall

let expectations = [
                       { request  = Mock.HttpRequest::{ method  = Some Mock.HttpMethod.GET
                                                      , path    = Some "/greet/pwet"
                                                      }
                       , response = Mock.HttpResponse::{ statusCode   = Mock.statusCreated
                                                       , body         = Some "Hello, pwet ! Comment que ca biche ?"
                                                       }
                      }
                      ,{ request  = Mock.HttpRequest::{ method  = Some Mock.HttpMethod.GET
                                                      , path    = Some "/greet/wololo"
                                                      }
                      , response = Mock.HttpResponse::{ statusCode   = Mock.statusOK
                                                      , body         = Some "Hello, Wololo !"
                                                      }
                      }
                   ]

in expectations
