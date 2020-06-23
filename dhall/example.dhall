let map = https://prelude.dhall-lang.org/List/map

let JSON = https://prelude.dhall-lang.org/JSON/package.dhall sha256:79dfc281a05bc7b78f927e0da0c274ee5709b1c55c9e5f59499cb28e9d6f3ec0
let Mock = https://raw.githubusercontent.com/dhall-mock/dhall-mock/master/dhall/Mock/package.dhall sha256:55595efe4300f51236f725146b98c16776ed8d37887a36b29b76c784b730a213

let User : Type =
    { userId        : Text
    , username      : Text
    , createdDate   : Text
    , lastLoginDate : Text
    }

let users = [ { userId        = "7e38a4e7-5bd8-4011-9391-98ffdca58c06"
              , username      = "jean_michel"
              , createdDate   = "2020-04-05T08:00:34"
              , lastLoginDate = "2020-04-05T08:00:34"
              },
              { userId        = "4c30413c-f04c-4a1f-ad22-6d8b9e84275b"
              , username      = "robert"
              , createdDate   = "2020-01-03T17:16:24"
              , lastLoginDate = "2020-04-04T13:00:02"
              },
              { userId        = "4c30413c-f04c-4a1f-ad22-6d8b9e84275b"
              , username      = "gérard"
              , createdDate   = "2019-08-04T11:12:45"
              , lastLoginDate = "2020-04-05T10:15:09"
              }
            ]

let mkJsonUserBody = \(u : User) ->
    JSON.object [ { mapKey = "userId"       , mapValue = JSON.string u.userId }
                , { mapKey = "username"     , mapValue = JSON.string u.username }
                , { mapKey = "createdDate"  , mapValue = JSON.string u.createdDate }
                , { mapKey = "lastLoginDate", mapValue = JSON.string u.lastLoginDate }
                ]

let mkUserExpectation = \(user: User) ->
        { request = 
             Mock.HttpRequest::{ method  = Some Mock.HttpMethod.GET
                               , path    = Some "/users/${user.userId}"
                               , headers = [ Mock.contentTypeJSON ]
                               }
        , response = 
             Mock.HttpResponse::{ statusCode = Mock.statusOK
                                , body       = Some (JSON.render (mkJsonUserBody user))
                                , headers    = [ Mock.contentTypeJSON ]
                                }
        }

in map User Mock.Expectation mkUserExpectation users
