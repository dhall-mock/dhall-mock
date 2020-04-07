let Mock = ./dhall/Mock/package.dhall

let expectations = [
                       { request  = { method  = Some Mock.HttpMethod.GET
                                   , path    = Some "/greet/pwet"
                                   }
                       , response = { statusCode   = Some +200
                                       , statusReason = None Text
                                       , body         = Some "Hello, pwet !"
                                       }
                      }
                      ,{ request  = { method  = Some Mock.HttpMethod.GET
                                   , path    = Some "/greet/pwet"
                                   }
                      , response = { statusCode   = Some +200
                                   , statusReason = None Text
                                   , body         = Some "Hello, pwet !"
                                   }
                      }
                   ]

in expectations