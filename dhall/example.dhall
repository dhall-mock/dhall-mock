let map = https://prelude.dhall-lang.org/List/map

let XML = https://prelude.dhall-lang.org/XML/package.dhall

let Mock = https://raw.githubusercontent.com/dhall-mock/dhall-mock/master/dhall/Mock/package.dhall

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

let mkUserBody = \(u : User) ->
    XML.element { attributes = [ { mapKey = "userId"       , mapValue = u.userId }
                               , { mapKey = "username"     , mapValue = u.username }
                               , { mapKey = "createdDate"  , mapValue = u.createdDate }
                               , { mapKey = "lastLoginDate", mapValue = u.lastLoginDate }
                               ]
                , content = [] : List XML.Type
                , name = "user"
                }

let mkUserExpectation = \(user: User) ->
    { request  = { method  = Some Mock.HttpMethod.GET
                 , path    = Some "/users/${user.userId}"
                 }
    , response = { statusCode   = Some +200
                 , statusReason = None Text
                 , body         = Some (XML.render (mkUserBody user))
                 }
    }

in map User Mock.Expectation mkUserExpectation users
