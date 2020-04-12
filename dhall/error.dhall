let Mock = ./dhall/Mock/package.dhall

let expectations = [
                       { request  = { method  = Some Mock.HttpMethod.GET
                                   , path    = Some "/greet/warcraft3"
                                   }
                       , response = { statusCode   = Some +200
                                       , statusReason = None Text
                                       , body         = Some "Oui monseigneur"
                                       }
                      }
                   ]

in expectation