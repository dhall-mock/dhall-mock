let Mock = ./dhall/Mock/package.dhall

let expectations = [
                       { request  = { method  = Some Mock.HttpMethod.GET
                                   , path    = Some "/greet/pwet"
                                   }
                       , response = { statusCode   = Some +201
                                       , statusReason = None Text
                                       , body         = Some "Hello, pwet ! Comment que ca biche ?"
                                       }
                      }
                      ,{ request  = { method  = Some Mock.HttpMethod.GET
                                   , path    = Some "/greet/wololo"
                                   }
                      , response = { statusCode   = Some +200
                                   , statusReason = None Text
                                   , body         = Some "Hello, Wololo !"
                                   }
                      }
                   ]

in expectations
