# Meta Pixel in JavaScript Events (Reference)

Source: https://support.gingrapp.com/hc/en-us/articles/25846641444109-Meta-Pixel-in-JavaScript-Events-Reference
Section ID: 25380161804813

Introduction

 Below is an example outlining the code you'll want to add to the  Customer app footer  field on the  System-wide Settings  page to notify Facebook Pixel when a customer requests a reservation in Gingr. 

 Reference: You'll first need to configure Meta Pixel ( https://www.facebook.com/business/a/facebook-pixel ). 

 Reference: You'll want to replace MY_FB_PIXEL_ID in the below example with the Pixel ID provided to you by Facebook. Alternatively, you can visit your Meta Pixel Dashboard to retrieve the code specific to your account. 

 Example

 Js

 < ! -- Facebook Pixel Code -- > 
 < script > 
 ! function ( f , b , e , v , n , t , s ) { if ( f . fbq ) return ; n = f . fbq = function ( ) { n . callMethod ? 
n . callMethod . apply ( n , arguments ) : n . queue . push ( arguments ) } ; if ( ! f . _fbq ) f . _fbq = n ; 
n . push = n ; n . loaded = ! 0 ; n . version = '2.0' ; n . queue = [ ] ; t = b . createElement ( e ) ; t . async = ! 0 ; 
t . src = v ; s = b . getElementsByTagName ( e ) [ 0 ] ; s . parentNode . insertBefore ( t , s ) } ( window , 
document , 'script' , '//connect.facebook.net/en_US/fbevents.js' ) ; 

 fbq ( 'init' , 'MY_FB_PIXEL_ID' ) ; 
 < / script > 

 < noscript > < img height = "1" width = "1" style = "display:none" 
src = "https://www.facebook.com/tr?id=MY_FB_PIXEL_ID&ev=Reservation&noscript=1" 
 / > < / noscript > 
 < ! -- End Facebook Pixel Code -- > 

 < script > 
document . addEventListener ( 'reservation_created' , function ( e ) { 
 fbq ( 'track' , 'Lead' , { } ) ; 
 } , false ) ; 
 < / script > 

 Related Resources

 Custom CSS & JavaScript   Topic Outline

 Add Custom CSS & JavaScript How-To

 JavaScript Events Emitted Reference

 Listen for Custom JavaScript Events Reference

 Google Tag Manager (Adwords/Analytics) in JavaScript Reference
