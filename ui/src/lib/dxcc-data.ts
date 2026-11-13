/**
 * Prefix table: which country (DXCC entity) a callsign belongs to, and a point for the direction and
 * the distance. Written by hand for CWLD (no table of another program was copied), so it can have gaps.
 *
 * Format of every line:   name|continent|latitude|longitude|prefixes
 *   latitude: degrees, north positive. longitude: degrees, east positive. The point is the middle of the
 *   country, good for a direction and a distance to a few hundred km, no more.
 *   prefixes: separated by commas. `[A-R]` stands for one letter of that range, `?` for any letter,
 *   `=DL1ABC` for one exact callsign. The longest match wins, so `KH6` beats `K`.
 *
 * To bring the table up to date run `node tools/update_prefixes.mjs cty.dat` (see that file). The list of
 * DXCC entities, special prefixes and portable callsigns changes now and then, so check it once a year.
 */
export const PREFIX_TABLE_VERSION = '2026-11-13 (hand written, 0.20.4)';

export const PREFIX_TABLE = `
Belgium|EU|50.8|4.4|O[N-T]
Netherlands|EU|52.2|5.5|P[A-I]
Germany|EU|51.0|10.0|D[A-R]
France|EU|46.5|2.5|F,HW,HX,HY,TH,TM,TP,TQ,TV
Luxembourg|EU|49.8|6.1|LX
England|EU|52.5|-1.5|G,M,2E
Scotland|EU|56.5|-4.2|GM,MM,2M
Wales|EU|52.3|-3.7|GW,MW,2W
Northern Ireland|EU|54.6|-6.7|GI,MI,2I
Isle of Man|EU|54.2|-4.5|GD,MD,2D
Jersey|EU|49.2|-2.1|GJ,MJ,2J
Guernsey|EU|49.5|-2.6|GU,MU,2U
Ireland|EU|53.3|-7.6|EI,EJ
Spain|EU|40.0|-3.7|E[A-H],A[M-O]
Balearic Islands|EU|39.6|3.0|EA6,EB6,EC6,ED6,EE6,EF6,EG6,EH6
Canary Islands|AF|28.3|-16.0|EA8,EB8,EC8,ED8,EE8,EF8,EG8,EH8
Ceuta and Melilla|AF|35.9|-5.3|EA9,EB9,EC9,ED9,EE9,EF9,EG9,EH9
Portugal|EU|39.6|-8.0|CQ,CR,CS,CT
Madeira|AF|32.7|-17.0|CQ3,CR3,CS3,CT3
Azores|EU|38.7|-28.0|CU,CQ2,CR2,CS2,CT2
Italy|EU|42.8|12.5|I
Sardinia|EU|40.0|9.0|IS0,IM0
Vatican|EU|41.9|12.5|HV
San Marino|EU|43.9|12.5|T7
Malta|EU|35.9|14.4|9H
Switzerland|EU|46.8|8.2|HB,HE
Liechtenstein|EU|47.1|9.5|HB0
Austria|EU|47.5|14.0|OE
Denmark|EU|56.0|10.0|OU,OV,OW,OZ,5P,5Q
Faroe Islands|EU|62.0|-6.8|OY
Greenland|NA|72.0|-40.0|OX,XP
Norway|EU|61.0|9.0|L[A-N]
Svalbard|EU|78.0|16.0|JW
Jan Mayen|EU|71.0|-8.3|JX
Bouvet|AF|-54.4|3.4|3Y
Sweden|EU|60.0|15.0|S[A-M],7S,8S
Finland|EU|62.0|26.0|O[F-J]
Aland Islands|EU|60.2|20.0|OH0
Market Reef|EU|60.3|19.1|OJ0
Iceland|EU|65.0|-19.0|TF
Poland|EU|52.0|19.5|S[N-R],HF,3Z
Czech Republic|EU|49.8|15.5|OK,OL
Slovak Republic|EU|48.7|19.5|OM
Hungary|EU|47.2|19.5|HA,HG
Romania|EU|45.9|25.0|Y[O-R]
Bulgaria|EU|42.7|25.5|LZ
Greece|EU|39.0|22.0|S[V-Z],J4
Crete|EU|35.2|24.9|SV9,J49
Dodecanese|AS|36.4|28.0|SV5,J45
Turkey|AS|39.0|35.0|T[A-C],YM
Croatia|EU|45.2|15.5|9A
Slovenia|EU|46.1|14.8|S5
Bosnia-Herzegovina|EU|44.0|17.8|E7,T9
Serbia|EU|44.0|21.0|YT,YU
Montenegro|EU|42.7|19.3|4O
North Macedonia|EU|41.6|21.7|Z3
Albania|EU|41.1|20.0|ZA
Kosovo|EU|42.6|20.9|Z6
Moldova|EU|47.2|28.5|ER
Ukraine|EU|49.0|31.5|UR,US,UT,UU,UV,UW,UX,UY,UZ,EM,EN,EO
Belarus|EU|53.7|28.0|EU,EV,EW
Lithuania|EU|55.3|24.0|LY
Latvia|EU|56.9|24.6|YL
Estonia|EU|58.7|25.5|ES
European Russia|EU|55.8|37.6|R,U[A-I]
Kaliningrad|EU|54.7|20.5|R2,R[A-Z]2,U[A-I]2
Asiatic Russia|AS|58.0|82.0|R[890],R[A-Z][890],U[A-I][890]
Andorra|EU|42.5|1.5|C3
Monaco|EU|43.7|7.4|3A
Gibraltar|EU|36.1|-5.4|ZB
Corsica|EU|42.0|9.0|TK
Cyprus|AS|35.0|33.0|5B,C4,H2,P3
UK Bases on Cyprus|AS|35.0|33.8|ZC4
Armenia|AS|40.2|44.5|EK
Georgia|AS|42.0|43.5|4L
Azerbaijan|AS|40.4|47.7|4J,4K
Israel|AS|31.5|35.0|4X,4Z
Jordan|AS|31.0|36.0|JY
Lebanon|AS|33.9|35.8|OD
Syria|AS|35.0|38.0|YK,6C
Iraq|AS|33.0|44.0|YI,HN
Iran|AS|32.0|53.0|E[P-Q],9B,9C,9D
Saudi Arabia|AS|24.0|45.0|HZ,7Z,8Z
Kuwait|AS|29.3|47.7|9K
Bahrain|AS|26.1|50.5|A9
Qatar|AS|25.3|51.2|A7
United Arab Emirates|AS|24.0|54.0|A6
Oman|AS|21.5|56.0|A4
Yemen|AS|15.5|48.0|7O
Afghanistan|AS|33.9|66.0|YA,T6
Pakistan|AS|30.0|70.0|A[P-S],6[P-S]
India|AS|22.0|79.0|V[U-W],AT,AU,AV,AW,8T,8U,8V,8W,8X,8Y
Sri Lanka|AS|7.5|80.7|4[P-S]
Nepal|AS|28.0|84.0|9N
Bhutan|AS|27.4|90.5|A5
Bangladesh|AS|23.8|90.4|S2,S3
Maldives|AS|3.2|73.2|8Q
Myanmar|AS|21.0|96.0|X[Y-Z],1Z
Thailand|AS|15.0|101.0|HS,E2
Laos|AS|18.0|103.0|XW
Cambodia|AS|12.5|105.0|XU
Vietnam|AS|16.0|107.0|3W,XV
Malaysia West|AS|3.5|102.0|9M2,9M4,9W2,9W4
Malaysia East|OC|3.0|113.0|9M6,9M8,9W6,9W8
Singapore|AS|1.3|103.8|9V,S6
Brunei|OC|4.5|114.7|V8
Indonesia|OC|-5.0|115.0|Y[B-H],7[A-I],8[A-I]
East Timor|OC|-8.8|125.9|4W
Philippines|OC|13.0|122.0|D[U-Z],4D,4E,4F,4G,4H,4I
China|AS|35.0|103.0|B
Taiwan|AS|23.7|121.0|B[M-Q],B[U-X]
Hong Kong|AS|22.3|114.2|VR
Macao|AS|22.2|113.5|XX9
Mongolia|AS|47.0|105.0|J[T-V]
North Korea|AS|40.0|127.0|HM,P5
South Korea|AS|36.5|127.8|HL,D[S-T],6K,6L,6M,6N,D7,D8,D9
Japan|AS|36.0|138.0|J[A-S],7[J-N],8[J-N]
Ogasawara|OC|27.1|142.2|JD1
Kazakhstan|AS|48.0|67.0|U[N-Q]
Uzbekistan|AS|41.5|64.5|U[J-M]
Turkmenistan|AS|39.0|59.0|EZ
Tajikistan|AS|38.8|71.0|EY
Kyrgyzstan|AS|41.5|75.0|EX
United States|NA|39.0|-98.0|K,N,W,A[A-K]
Alaska|NA|64.0|-153.0|KL,AL,NL,WL
Hawaii|OC|20.7|-156.5|KH6,KH7,AH6,AH7,NH6,NH7,WH6,WH7
Guam|OC|13.4|144.8|KH2,AH2,NH2,WH2
Northern Mariana Islands|OC|15.2|145.7|KH0,AH0,NH0,WH0
American Samoa|OC|-14.3|-170.7|KH8,AH8,NH8,WH8
Wake Island|OC|19.3|166.6|KH9,AH9,NH9,WH9
Midway Island|OC|28.2|-177.4|KH4,AH4,NH4,WH4
Johnston Island|OC|16.7|-169.5|KH3,AH3,NH3,WH3
Puerto Rico|NA|18.2|-66.5|KP3,KP4,NP3,NP4,WP3,WP4
US Virgin Islands|NA|18.3|-64.9|KP2,NP2,WP2
Navassa Island|NA|18.4|-75.0|KP1,NP1,WP1
Guantanamo Bay|NA|19.9|-75.2|KG4
Canada|NA|56.0|-96.0|V[A-G],VO,VY,X[J-O],C[F-K],CY,CZ
Sable Island|NA|43.9|-60.0|CY0
St Paul Island|NA|47.2|-60.1|CY9
St Pierre and Miquelon|NA|46.8|-56.3|FP
Mexico|NA|23.0|-102.0|X[A-I],4[A-C],6[D-J]
Guatemala|NA|15.5|-90.3|TD,TG
Belize|NA|17.2|-88.7|V3
El Salvador|NA|13.8|-88.9|HU,YS
Honduras|NA|14.8|-86.6|HQ,HR
Nicaragua|NA|12.9|-85.0|H6,H7,HT,YN
Costa Rica|NA|10.0|-84.0|TE,TI
Panama|NA|8.5|-80.0|H[O-P],H3,H8,H9,3E,3F
Cuba|NA|21.8|-79.5|CL,CM,CO,T4
Jamaica|NA|18.1|-77.3|6Y
Haiti|NA|19.0|-72.5|HH,4V
Dominican Republic|NA|19.0|-70.5|HI
Bahamas|NA|24.5|-77.5|C6
Turks and Caicos|NA|21.8|-71.8|VP5
Cayman Islands|NA|19.3|-81.2|ZF
Bermuda|NA|32.3|-64.8|VP9
Anguilla|NA|18.2|-63.1|VP2E
Montserrat|NA|16.7|-62.2|VP2M
British Virgin Islands|NA|18.4|-64.6|VP2V
Antigua and Barbuda|NA|17.1|-61.8|V2
St Kitts and Nevis|NA|17.3|-62.7|V4
Dominica|NA|15.4|-61.4|J7
St Lucia|NA|13.9|-61.0|J6
St Vincent|NA|13.2|-61.2|J8
Grenada|NA|12.1|-61.7|J3
Barbados|NA|13.2|-59.5|8P
Trinidad and Tobago|SA|10.5|-61.3|9Y,9Z
Martinique|NA|14.7|-61.0|FM
Guadeloupe|NA|16.2|-61.6|FG
St Barthelemy|NA|17.9|-62.8|FJ
St Martin|NA|18.1|-63.0|FS
Aruba|SA|12.5|-70.0|P4
Curacao|SA|12.2|-69.0|PJ2
Bonaire|SA|12.2|-68.3|PJ4
Sint Maarten|NA|18.0|-63.1|PJ7
Venezuela|SA|7.0|-66.0|Y[V-Y],4M
Colombia|SA|4.0|-73.0|H[J-K],5J,5K
Ecuador|SA|-1.5|-78.5|H[C-D]
Peru|SA|-9.0|-75.0|O[A-C],4T
Brazil|SA|-10.0|-52.0|P[P-Y],Z[V-Z]
Fernando de Noronha|SA|-3.9|-32.4|PY0F,PP0F,PU0F
Bolivia|SA|-17.0|-64.7|CP
Paraguay|SA|-23.4|-58.4|ZP
Uruguay|SA|-33.0|-56.0|C[V-X]
Argentina|SA|-34.0|-64.0|LO,LP,LQ,LR,LS,LT,LU,LV,LW,L[2-9],AY,AZ
Chile|SA|-33.5|-70.7|C[A-E],X[Q-R],3G
Easter Island|SA|-27.1|-109.4|CE0Y
Juan Fernandez|SA|-33.6|-78.8|CE0Z
Falkland Islands|SA|-51.7|-59.0|VP8
Guyana|SA|5.0|-59.0|8R
Suriname|SA|4.0|-56.0|PZ
French Guiana|SA|4.0|-53.0|FY
Australia|OC|-25.0|134.0|V[H-N],AX
Lord Howe Island|OC|-31.5|159.1|VK9L
Norfolk Island|OC|-29.0|168.0|VK9N
Christmas Island|OC|-10.5|105.7|VK9X
Cocos (Keeling) Islands|OC|-12.2|96.8|VK9C
Macquarie Island|OC|-54.6|158.9|VK0M
Heard Island|AF|-53.1|73.5|VK0H
New Zealand|OC|-41.0|174.0|Z[K-M],ZL
Chatham Islands|OC|-44.0|-176.5|ZL7
Kermadec Islands|OC|-29.2|-177.9|ZL8
Papua New Guinea|OC|-6.5|147.0|P2
Solomon Islands|OC|-9.0|160.0|H4
Vanuatu|OC|-17.5|168.0|YJ
New Caledonia|OC|-21.5|165.5|FK
Fiji|OC|-18.0|178.0|3D2
Tonga|OC|-21.2|-175.2|A3
Samoa|OC|-13.8|-172.0|5W
Tuvalu|OC|-8.5|179.2|T2
Kiribati West|OC|1.4|173.0|T30
Kiribati Central|OC|-3.0|-171.7|T31
Kiribati East|OC|1.9|-157.4|T32
Nauru|OC|-0.5|166.9|C2
Marshall Islands|OC|7.1|171.2|V7
Micronesia|OC|6.9|158.2|V6
Palau|OC|7.5|134.6|T8
Cook Islands|OC|-21.2|-159.8|E5,ZK1
Niue|OC|-19.1|-169.9|E6,ZK2
Tokelau|OC|-9.2|-171.8|ZK3
French Polynesia|OC|-17.6|-149.5|FO
Wallis and Futuna|OC|-13.3|-176.2|FW
Pitcairn|OC|-25.1|-130.1|VP6
Egypt|AF|27.0|30.0|SS,SU,6A,6B
Libya|AF|27.0|17.0|5A
Tunisia|AF|34.0|9.0|3V,TS
Algeria|AF|28.0|2.6|7R,7T,7U,7V,7W,7X,7Y
Morocco|AF|32.0|-6.0|CN,5C,5D,5E,5F,5G
Western Sahara|AF|24.5|-13.0|S0
Mauritania|AF|20.0|-10.5|5T
Senegal|AF|14.5|-14.5|6V,6W
Gambia|AF|13.5|-15.5|C5
Guinea-Bissau|AF|12.0|-15.0|J5
Guinea|AF|10.5|-11.0|3X
Sierra Leone|AF|8.5|-11.8|9L
Liberia|AF|6.5|-9.5|EL,A8,D5,5L,5M
Ivory Coast|AF|7.5|-5.5|TU
Burkina Faso|AF|12.3|-1.5|XT
Mali|AF|17.0|-4.0|TZ
Niger|AF|16.0|8.0|5U
Ghana|AF|7.9|-1.0|9G
Togo|AF|8.6|1.0|5V
Benin|AF|9.5|2.3|TY
Nigeria|AF|9.5|8.0|5N,5O
Cameroon|AF|5.7|12.7|TJ
Chad|AF|15.0|19.0|TT
Central African Republic|AF|6.5|20.5|TL
Equatorial Guinea|AF|1.7|10.3|3C
Gabon|AF|-0.8|11.6|TR
Congo|AF|-1.0|15.8|TN
DR Congo|AF|-3.0|23.0|9[Q-T]
Angola|AF|-12.0|17.5|D2,D3
Sao Tome and Principe|AF|0.3|6.7|S9
Cape Verde|AF|16.0|-24.0|D4
Sudan|AF|15.5|30.0|ST,6T,6U
South Sudan|AF|7.0|30.0|Z8
Ethiopia|AF|9.0|39.5|ET,9E,9F
Eritrea|AF|15.3|39.0|E3
Palestine|AS|31.9|35.2|E4
Djibouti|AF|11.8|42.6|J2
Somalia|AF|5.0|46.0|T5,6O
Kenya|AF|0.0|38.0|5Y,5Z
Uganda|AF|1.3|32.4|5X
Tanzania|AF|-6.5|35.0|5H,5I
Rwanda|AF|-2.0|29.8|9X
Burundi|AF|-3.4|29.9|9U
Zambia|AF|-14.0|28.0|9J,9I
Malawi|AF|-13.5|34.0|7Q
Mozambique|AF|-18.5|35.0|C8,C9
Zimbabwe|AF|-19.0|30.0|Z2
Botswana|AF|-22.0|24.0|A2,8O
Namibia|AF|-22.0|17.0|V5
South Africa|AF|-29.0|24.0|Z[R-U],ZS,H5,S4,S8
Lesotho|AF|-29.6|28.2|7P
Eswatini|AF|-26.5|31.5|3DA
Madagascar|AF|-19.0|47.0|5R,5S,6X
Mauritius|AF|-20.3|57.6|3B8
Rodrigues|AF|-19.7|63.4|3B9
Agalega|AF|-10.4|56.6|3B6,3B7
Reunion|AF|-21.1|55.5|FR
Mayotte|AF|-12.8|45.2|FH
Comoros|AF|-11.7|43.3|D6
Seychelles|AF|-4.7|55.5|S7
Chagos|AF|-7.3|72.4|VQ9
St Helena|AF|-15.9|-5.7|ZD7
Ascension Island|AF|-7.9|-14.4|ZD8
Tristan da Cunha|AF|-37.1|-12.3|ZD9
Kerguelen|AF|-49.3|69.5|FT5X
Crozet|AF|-46.4|51.8|FT5W
Amsterdam and St Paul|AF|-37.8|77.5|FT5Z
Antarctica|SA|-75.0|0.0|CE9,KC4,8J1
United Nations HQ|NA|40.7|-74.0|4U1UN
ITU Geneva|EU|46.2|6.1|4U1ITU
Sovereign Military Order of Malta|EU|41.9|12.5|1A
`.trim();
