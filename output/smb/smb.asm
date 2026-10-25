; 字节级往返反汇编（Feature 019），由 nesc disasm-full 生成
; 重汇编：nesc asm <本文件> → 与原 ROM 逐字节一致
.ines prg=2 chr=1 mapper=0 flags6=$01
.chr "smb.chr"
.org $8000
Reset:
    SEI
    CLD
    LDA #$10
    STA $2000
    LDX #$FF
    TXS
L800A:
    LDA $2002
    BPL L800A
L800F:
    LDA $2002
    BPL L800F
    LDY #$FE
    LDX #$05
L8018:
    LDA $07D7,X
    CMP #$0A
    BCS L802B
    DEX
    BPL L8018
    LDA $07FF
    CMP #$A5
    BNE L802B
    LDY #$D6
L802B:
    JSR L90CC
    STA $4011
    STA $0770
    LDA #$A5
    STA $07FF
    STA $07A7
    LDA #$0F
    STA $4015
    LDA #$06
    STA $2001
    JSR L8220
    JSR L8E19
    INC $0774
    LDA $0778
    ORA #$80
    JSR L8EED
L8057:
    JMP L8057
    .byte $01, $A4, $C8, $EC, $10, $00, $41, $41, $4C, $34, $3C, $44, $54, $68, $7C, $A8
    .byte $BF, $DE, $EF, $03, $8C, $8C, $8C, $8D, $03, $03, $03, $8D, $8D, $8D, $8D, $8D
    .byte $8D, $8D, $8D, $8D, $8D, $8D, $00, $40
NMI:
    LDA $0778
    AND #$7F
    STA $0778
    AND #$7E
    STA $2000
    LDA $0779
    AND #$E6
    LDY $0774
    BNE L809E
    LDA $0779
    ORA #$1E
L809E:
    STA $0779
    AND #$E7
    STA $2001
    LDX $2002
    LDA #$00
    JSR L8EE6
    STA $2003
    LDA #$02
    STA $4014
    LDX $0773
    LDA $805A,X
    STA $00
    LDA $806D,X
    STA $01
    JSR L8EDD
    LDY #$00
    LDX $0773
    CPX #$06
    BNE L80D0
    INY
L80D0:
    LDX $8080,Y
    LDA #$00
    STA $0300,X
    STA $0301,X
    STA $0773
    LDA $0779
    STA $2001
    JSR LF2D0
    JSR L8E5C
    JSR L8182
    JSR L8F97
    LDA $0776
    LSR A
    BCS L811B
    LDA $0747
    BEQ L8100
    DEC $0747
    BNE L8119
L8100:
    LDX #$14
    DEC $077F
    BPL L810E
    LDA #$14
    STA $077F
    LDX #$23
L810E:
    LDA $0780,X
    BEQ L8116
    DEC $0780,X
L8116:
    DEX
    BPL L810E
L8119:
    INC $09
L811B:
    LDX #$00
    LDY #$07
    LDA $07A7
    AND #$02
    STA $00
    LDA $07A8
    AND #$02
    EOR $00
    CLC
    BEQ L8131
    SEC
L8131:
    ROR $07A7,X
    INX
    DEY
    BNE L8131
    LDA $0722
    BEQ L815C
L813D:
    LDA $2002
    AND #$40
    BNE L813D
    LDA $0776
    LSR A
    BCS L8150
    JSR L8223
    JSR L81C6
L8150:
    LDA $2002
    AND #$40
    BEQ L8150
    LDY #$14
L8159:
    DEY
    BNE L8159
L815C:
    LDA $073F
    STA $2005
    LDA $0740
    STA $2005
    LDA $0778
    PHA
    STA $2000
    LDA $0776
    LSR A
    BCS L8178
    JSR L8212
L8178:
    LDA $2002
    PLA
    ORA #$80
    STA $2000
    RTI
L8182:
    LDA $0770
    CMP #$02
    BEQ L8194
    CMP #$01
    BNE L81C5
    LDA $0772
    CMP #$03
    BNE L81C5
L8194:
    LDA $0777
    BEQ L819D
    DEC $0777
    RTS
L819D:
    LDA $06FC
    AND #$10
    BEQ L81BD
    LDA $0776
    AND #$80
    BNE L81C5
    LDA #$2B
    STA $0777
    LDA $0776
    TAY
    INY
    STY $FA
    EOR #$01
    ORA #$80
    BNE L81C2
L81BD:
    LDA $0776
    AND #$7F
L81C2:
    STA $0776
L81C5:
    RTS
L81C6:
    LDY $074E
    LDA #$28
    STA $00
    LDX #$0E
L81CF:
    LDA $06E4,X
    CMP $00
    BCC L81E5
    LDY $06E0
    CLC
    ADC $06E1,Y
    BCC L81E2
    CLC
    ADC $00
L81E2:
    STA $06E4,X
L81E5:
    DEX
    BPL L81CF
    LDX $06E0
    INX
    CPX #$03
    BNE L81F2
    LDX #$00
L81F2:
    STX $06E0
    LDX #$08
    LDY #$02
L81F9:
    LDA $06E9,Y
    STA $06F1,X
    CLC
    ADC #$08
    STA $06F2,X
    CLC
    ADC #$08
    STA $06F3,X
    DEX
    DEX
    DEX
    DEY
    BPL L81F9
    RTS
L8212:
    LDA $0770
    JSR L8E04
    .word L8231
    .word LAEDC
    .word L838B
    .word L9218
L8220:
    LDY #$00
    .byte $2C
L8223:
    LDY #$04
    LDA #$F8
L8227:
    STA $0200,Y
    INY
    INY
    INY
    INY
    BNE L8227
    RTS
L8231:
    LDA $0772
    JSR L8E04
    .word L8FCF
    .word L8567
    .word L9061
    .word L8245
    .byte $04, $20, $73, $01, $00, $00
L8245:
    LDY #$00
    LDA $06FC
    ORA $06FD
    CMP #$10
    BEQ L8255
    CMP #$90
    BNE L8258
L8255:
    JMP L82D8
L8258:
    CMP #$20
    BEQ L8276
    LDX $07A2
    BNE L826C
    STA $0780
    JSR L836B
    BCS L82C9
    JMP L82C0
L826C:
    LDX $07FC
    BEQ L82BB
    CMP #$40
    BNE L82BB
    INY
L8276:
    LDA $07A2
    BEQ L82C9
    LDA #$18
    STA $07A2
    LDA $0780
    BNE L82BB
    LDA #$10
    STA $0780
    CPY #$01
    BEQ L829C
    LDA $077A
    EOR #$01
    STA $077A
    JSR L8325
    JMP L82BB
L829C:
    LDX $076B
    INX
    TXA
    AND #$07
    STA $076B
    JSR L830E
L82A9:
    LDA $823F,X
    STA $0300,X
    INX
    CPX #$06
    BMI L82A9
    LDY $075F
    INY
    STY $0304
L82BB:
    LDA #$00
    STA $06FC
L82C0:
    JSR LAEEA
    LDA $0E
    CMP #$06
    BNE L830D
L82C9:
    LDA #$00
    STA $0770
    STA $0772
    STA $0722
    INC $0774
    RTS
L82D8:
    LDY $07A2
    BEQ L82C9
    ASL A
    BCC L82E6
    LDA $07FD
    JSR L830E
L82E6:
    JSR L9C03
    INC $075D
    INC $0764
    INC $0757
    INC $0770
    LDA $07FC
    STA $076A
    LDA #$00
    STA $0772
    STA $07A2
    LDX #$17
    LDA #$00
L8307:
    STA $07DD,X
    DEX
    BPL L8307
L830D:
    RTS
L830E:
    STA $075F
    STA $0766
    LDX #$00
    STX $0760
    STX $0767
    RTS
    .byte $07, $22, $49, $83, $CE, $24, $24, $00
L8325:
    LDY #$07
L8327:
    LDA $831D,Y
    STA $0300,Y
    DEY
    BPL L8327
    LDA $077A
    BEQ L833F
    LDA #$24
    STA $0304
    LDA #$CE
    STA $0306
L833F:
    RTS
    .byte $01, $80, $02, $81, $41, $80, $01, $42, $C2, $02, $80, $41, $C1, $41, $C1, $01
    .byte $C1, $01, $02, $80, $00, $9B, $10, $18, $05, $2C, $20, $24, $15, $5A, $10, $20
    .byte $28, $30, $20, $10, $80, $20, $30, $30, $01, $FF, $00
L836B:
    LDX $0717
    LDA $0718
    BNE L8380
    INX
    INC $0717
    SEC
    LDA $8354,X
    STA $0718
    BEQ L838A
L8380:
    LDA $833F,X
    STA $06FC
    DEC $0718
    CLC
L838A:
    RTS
L838B:
    JSR L83A0
    LDA $0772
    BEQ L839A
    LDX #$00
    STX $08
    JSR LC047
L839A:
    JSR LF12A
    JMP LEEE9
L83A0:
    LDA $0772
    JSR L8E04
    .word LCFEC
    .word L83B0
    .word L83BD
    .word L83F6
    .word L8461
L83B0:
    LDX $071B
    INX
    STX $34
    LDA #$08
    STA $FC
    JMP L874E
L83BD:
    LDY #$00
    STY $35
    LDA $6D
    CMP $34
    BNE L83CD
    LDA $86
    CMP #$60
    BCS L83D0
L83CD:
    INC $35
    INY
L83D0:
    TYA
    JSR LB0E6
    LDA $071A
    CMP $34
    BEQ L83F1
    LDA $0768
    CLC
    ADC #$80
    STA $0768
    LDA #$01
    ADC #$00
    TAY
    JSR LAFC4
    JSR LAF6F
    INC $35
L83F1:
    LDA $35
    BEQ L845D
    RTS
L83F6:
    LDA $0749
    BNE L8443
    LDA $0719
    BEQ L8418
    CMP #$09
    BCS L8443
    LDY $075F
    CPY #$07
    BNE L8414
    CMP #$03
    BCC L8443
    SBC #$01
    JMP L8418
L8414:
    CMP #$02
    BCC L8443
L8418:
    TAY
    BNE L8423
    LDA $0753
    BEQ L8434
    INY
    BNE L8434
L8423:
    INY
    LDA $075F
    CMP #$07
    BEQ L8434
    DEY
    CPY #$04
    BCS L8456
    CPY #$03
    BCS L8443
L8434:
    CPY #$03
    BNE L843C
    LDA #$04
    STA $FC
L843C:
    TYA
    CLC
    ADC #$0C
    STA $0773
L8443:
    LDA $0749
    CLC
    ADC #$04
    STA $0749
    LDA $0719
    ADC #$00
    STA $0719
    CMP #$07
L8456:
    BCC L8460
    LDA #$06
    STA $07A1
L845D:
    INC $0772
L8460:
    RTS
L8461:
    LDA $07A1
    BNE L8486
    LDY $075F
    CPY #$07
    BCS L8487
    LDA #$00
    STA $0760
    STA $075C
    STA $0772
    INC $075F
    JSR L9C03
    INC $0757
    LDA #$01
    STA $0770
L8486:
    RTS
L8487:
    LDA $06FC
    ORA $06FD
    AND #$40
    BEQ L849E
    LDA #$01
    STA $07FC
    LDA #$FF
    STA $075A
    JSR L9248
L849E:
    RTS
    .byte $FF, $FF, $F6, $FB, $F7, $FB, $F8, $FB, $F9, $FB, $FA, $FB, $F6, $50, $F7, $50
    .byte $F8, $50, $F9, $50, $FA, $50, $FD, $FE, $FF, $41, $42, $44, $45, $48, $31, $32
    .byte $34, $35, $38, $00
L84C3:
    LDA $0110,X
    BEQ L8486
    CMP #$0B
    BCC L84D1
    LDA #$0B
    STA $0110,X
L84D1:
    TAY
    LDA $012C,X
    BNE L84DB
    STA $0110,X
    RTS
L84DB:
    DEC $012C,X
    CMP #$2B
    BNE L8500
    CPY #$0B
    BNE L84ED
    INC $075A
    LDA #$40
    STA $FE
L84ED:
    LDA $84B7,Y
    LSR A
    LSR A
    LSR A
    LSR A
    TAX
    LDA $84B7,Y
    AND #$0F
    STA $0134,X
    JSR LBC27
L8500:
    LDY $06E5,X
    LDA $16,X
    CMP #$12
    BEQ L852B
    CMP #$0D
    BEQ L852B
    CMP #$05
    BEQ L8523
    CMP #$0A
    BEQ L852B
    CMP #$0B
    BEQ L852B
    CMP #$09
    BCS L8523
    LDA $1E,X
    CMP #$02
    BCS L852B
L8523:
    LDX $03EE
    LDY $06EC,X
    LDX $08
L852B:
    LDA $011E,X
    CMP #$18
    BCC L8537
    SBC #$01
    STA $011E,X
L8537:
    LDA $011E,X
    SBC #$08
    JSR LE5C1
    LDA $0117,X
    STA $0203,Y
    CLC
    ADC #$08
    STA $0207,Y
    LDA #$02
    STA $0202,Y
    STA $0206,Y
    LDA $0110,X
    ASL A
    TAX
    LDA $849F,X
    STA $0201,Y
    LDA $84A0,X
    STA $0205,Y
    LDX $08
    RTS
L8567:
    LDA $073C
    JSR L8E04
    .word L858B
    .word L859B
    .word L8652
    .word L865A
    .word L8693
    .word L889D
    .word L86A8
    .word L889D
    .word L86E6
    .word L85BF
    .word L85E3
    .word L8643
    .word L86FF
    .word L8732
    .word L8749
L858B:
    JSR L8220
    JSR L8E19
    LDA $0770
    BEQ L85C8
    LDX #$03
    JMP L85C5
L859B:
    LDA $0744
    PHA
    LDA $0756
    PHA
    LDA #$00
    STA $0756
    LDA #$02
    STA $0744
    JSR L85F1
    PLA
    STA $0756
    PLA
    STA $0744
    JMP L8745
    .byte $01, $02, $03, $04
L85BF:
    LDY $074E
    LDX $85BB,Y
L85C5:
    STX $0773
L85C8:
    JMP L8745
    .byte $00, $09, $0A, $04, $22, $22, $0F, $0F, $0F, $22, $0F, $0F, $22, $16, $27, $18
    .byte $22, $30, $27, $19, $22, $37, $27, $16
L85E3:
    LDY $0744
    BEQ L85EE
    LDA $85C7,Y
    STA $0773
L85EE:
    INC $073C
L85F1:
    LDX $0300
    LDY #$00
    LDA $0753
    BEQ L85FD
    LDY #$04
L85FD:
    LDA $0756
    CMP #$02
    BNE L8606
    LDY #$08
L8606:
    LDA #$03
    STA $00
L860A:
    LDA $85D7,Y
    STA $0304,X
    INY
    INX
    DEC $00
    BPL L860A
    LDX $0300
    LDY $0744
    BNE L8621
    LDY $074E
L8621:
    LDA $85CF,Y
    STA $0304,X
    LDA #$3F
    STA $0301,X
    LDA #$10
    STA $0302,X
    LDA #$04
    STA $0303,X
    LDA #$00
    STA $0308,X
    TXA
    CLC
    ADC #$07
L863F:
    STA $0300
    RTS
L8643:
    LDA $0733
    CMP #$01
    BNE L864F
    LDA #$0B
L864C:
    STA $0773
L864F:
    JMP L8745
L8652:
    LDA #$00
    JSR L8808
    JMP L8745
L865A:
    JSR LBC30
    LDX $0300
    LDA #$20
    STA $0301,X
    LDA #$73
    STA $0302,X
    LDA #$03
    STA $0303,X
    LDY $075F
    INY
    TYA
    STA $0304,X
    LDA #$28
    STA $0305,X
    LDY $075C
    INY
    TYA
    STA $0306,X
    LDA #$00
    STA $0307,X
    TXA
    CLC
    ADC #$06
    STA $0300
    JMP L8745
L8693:
    LDA $0759
    BEQ L86A2
    LDA #$00
    STA $0759
    LDA #$02
    JMP L86C7
L86A2:
    INC $073C
    JMP L8745
L86A8:
    LDA $0770
    BEQ L86E0
    CMP #$03
    BEQ L86D3
    LDA $0752
    BNE L86E0
    LDY $074E
    CPY #$03
    BEQ L86C2
    LDA $0769
    BNE L86E0
L86C2:
    JSR LEFA4
    LDA #$01
L86C7:
    JSR L8808
    JSR L88A5
    LDA #$00
    STA $0774
    RTS
L86D3:
    LDA #$12
    STA $07A0
    LDA #$03
    JSR L8808
    JMP L874E
L86E0:
    LDA #$08
    STA $073C
    RTS
L86E6:
    INC $0774
L86E9:
    JSR L92B0
    LDA $071F
    BNE L86E9
    DEC $071E
    BPL L86F9
    INC $073C
L86F9:
    LDA #$06
    STA $0773
    RTS
L86FF:
    LDA $0770
    BNE L874E
    LDA #$1E
    STA $2006
    LDA #$C0
    STA $2006
    LDA #$03
    STA $01
    LDY #$00
    STY $00
    LDA $2007
L8719:
    LDA $2007
    STA ($00),Y
    INY
    BNE L8723
    INC $01
L8723:
    LDA $01
    CMP #$04
    BNE L8719
    CPY #$3A
    BCC L8719
    LDA #$05
    JMP L864C
L8732:
    LDA $0770
    BNE L874E
    LDX #$00
L8739:
    STA $0300,X
    STA $0400,X
    DEX
    BNE L8739
    JSR L8325
L8745:
    INC $073C
    RTS
L8749:
    LDA #$FA
    JSR LBC36
L874E:
    INC $0772
    RTS
    .byte $20, $43, $05, $16, $0A, $1B, $12, $18, $20, $52, $0B, $20, $18, $1B, $15, $0D
    .byte $24, $24, $1D, $12, $16, $0E, $20, $68, $05, $00, $24, $24, $2E, $29, $23, $C0
    .byte $7F, $AA, $23, $C2, $01, $EA, $FF, $21, $CD, $07, $24, $24, $29, $24, $24, $24
    .byte $24, $21, $4B, $09, $20, $18, $1B, $15, $0D, $24, $24, $28, $24, $22, $0C, $47
    .byte $24, $23, $DC, $01, $BA, $FF, $21, $CD, $05, $16, $0A, $1B, $12, $18, $22, $0C
    .byte $07, $1D, $12, $16, $0E, $24, $1E, $19, $FF, $21, $CD, $05, $16, $0A, $1B, $12
    .byte $18, $22, $0B, $09, $10, $0A, $16, $0E, $24, $18, $1F, $0E, $1B, $FF, $25, $84
    .byte $15, $20, $0E, $15, $0C, $18, $16, $0E, $24, $1D, $18, $24, $20, $0A, $1B, $19
    .byte $24, $23, $18, $17, $0E, $2B, $26, $25, $01, $24, $26, $2D, $01, $24, $26, $35
    .byte $01, $24, $27, $D9, $46, $AA, $27, $E1, $45, $AA, $FF, $15, $1E, $12, $10, $12
    .byte $04, $03, $02, $00, $24, $05, $24, $00, $08, $07, $06, $00, $00, $00, $27, $27
    .byte $46, $4E, $59, $61, $6E, $6E
L8808:
    PHA
    ASL A
    TAY
    CPY #$04
    BCC L881B
    CPY #$08
    BCC L8815
    LDY #$08
L8815:
    LDA $077A
    BNE L881B
    INY
L881B:
    LDX $87FE,Y
    LDY #$00
L8820:
    LDA $8752,X
    CMP #$FF
    BEQ L882E
    STA $0301,Y
    INX
    INY
    BNE L8820
L882E:
    LDA #$00
    STA $0301,Y
    PLA
    TAX
    CMP #$04
    BCS L8882
    DEX
    BNE L885F
    LDA $075A
    CLC
    ADC #$01
    CMP #$0A
    BCC L884D
    SBC #$0A
    LDY #$9F
    STY $0308
L884D:
    STA $0309
    LDY $075F
    INY
    STY $0314
    LDY $075C
    INY
    STY $0316
    RTS
L885F:
    LDA $077A
    BEQ L8881
    LDA $0753
    DEX
    BNE L8873
    LDY $0770
    CPY #$03
    BEQ L8873
    EOR #$01
L8873:
    LSR A
    BCC L8881
    LDY #$04
L8878:
    LDA $87ED,Y
    STA $0304,Y
    DEY
    BPL L8878
L8881:
    RTS
L8882:
    SBC #$04
    ASL A
    ASL A
    TAX
    LDY #$00
L8889:
    LDA $87F2,X
    STA $031C,Y
    INX
    INY
    INY
    INY
    INY
    CPY #$0C
    BCC L8889
    LDA #$2C
    JMP L863F
L889D:
    LDA $07A0
    BNE L88AD
    JSR L8220
L88A5:
    LDA #$07
    STA $07A0
    INC $073C
L88AD:
    RTS
L88AE:
    LDA $0726
    AND #$01
    STA $05
    LDY $0340
    STY $00
    LDA $0721
    STA $0342,Y
    LDA $0720
    STA $0341,Y
    LDA #$9A
    STA $0343,Y
    LDA #$00
    STA $04
    TAX
L88D0:
    STX $01
    LDA $06A1,X
    AND #$C0
    STA $03
    ASL A
    ROL A
    ROL A
    TAY
    LDA $8B08,Y
    STA $06
    LDA $8B0C,Y
    STA $07
    LDA $06A1,X
    ASL A
    ASL A
    STA $02
    LDA $071F
    AND #$01
    EOR #$01
    ASL A
    ADC $02
    TAY
    LDX $00
    LDA ($06),Y
    STA $0344,X
    INY
    LDA ($06),Y
    STA $0345,X
    LDY $04
    LDA $05
    BNE L891A
    LDA $01
    LSR A
    BCS L892A
    ROL $03
    ROL $03
    ROL $03
    JMP L8930
L891A:
    LDA $01
    LSR A
    BCS L892E
    LSR $03
    LSR $03
    LSR $03
    LSR $03
    JMP L8930
L892A:
    LSR $03
    LSR $03
L892E:
    INC $04
L8930:
    LDA $03F9,Y
    ORA $03
    STA $03F9,Y
    INC $00
    INC $00
    LDX $01
    INX
    CPX #$0D
    BCC L88D0
    LDY $00
    INY
    INY
    INY
    LDA #$00
    STA $0341,Y
    STY $0340
    INC $0721
    LDA $0721
    AND #$1F
    BNE L8967
    LDA #$80
    STA $0721
    LDA $0720
    EOR #$04
    STA $0720
L8967:
    JMP L89BD
L896A:
    LDA $0721
    AND #$1F
    SEC
    SBC #$04
    AND #$1F
    STA $01
    LDA $0720
    BCS L897D
    EOR #$04
L897D:
    AND #$04
    ORA #$23
    STA $00
    LDA $01
    LSR A
    LSR A
    ADC #$C0
    STA $01
    LDX #$00
    LDY $0340
L8990:
    LDA $00
    STA $0341,Y
    LDA $01
    CLC
    ADC #$08
    STA $0342,Y
    STA $01
    LDA $03F9,X
    STA $0344,Y
    LDA #$01
    STA $0343,Y
    LSR A
    STA $03F9,X
    INY
    INY
    INY
    INY
    INX
    CPX #$07
    BCC L8990
    STA $0341,Y
    STY $0340
L89BD:
    LDA #$06
    STA $0773
    RTS
    .byte $27, $27, $27, $17, $07, $17, $3F, $0C, $04, $FF, $FF, $FF, $FF, $00, $0F, $07
    .byte $12, $0F, $0F, $07, $17, $0F, $0F, $07, $17, $1C, $0F, $07, $17, $00
L89E1:
    LDA $09
    AND #$07
    BNE L8A38
    LDX $0300
    CPX #$31
    BCS L8A38
    TAY
L89EF:
    LDA $89C9,Y
    STA $0301,X
    INX
    INY
    CPY #$08
    BCC L89EF
    LDX $0300
    LDA #$03
    STA $00
    LDA $074E
    ASL A
    ASL A
    TAY
L8A08:
    LDA $89D1,Y
    STA $0304,X
    INY
    INX
    DEC $00
    BPL L8A08
    LDX $0300
    LDY $06D4
    LDA $89C3,Y
    STA $0305,X
    LDA $0300
    CLC
    ADC #$07
    STA $0300
    INC $06D4
    LDA $06D4
    CMP #$06
    BCC L8A38
    LDA #$00
    STA $06D4
L8A38:
    RTS
    .byte $45, $45, $47, $47, $47, $47, $47, $47, $57, $58, $59, $5A, $24, $24, $24, $24
    .byte $26, $26, $26, $26
L8A4D:
    LDY #$41
    LDA #$03
    LDX $074E
    BNE L8A58
    LDA #$04
L8A58:
    JSR L8A97
    LDA #$06
    STA $0773
    RTS
L8A61:
    JSR L8A6D
    INC $03F0
    DEC $03EC,X
    RTS
L8A6B:
    LDA #$00
L8A6D:
    LDY #$03
    CMP #$00
    BEQ L8A87
    LDY #$00
    CMP #$58
    BEQ L8A87
    CMP #$51
    BEQ L8A87
    INY
    CMP #$5D
    BEQ L8A87
    CMP #$52
    BEQ L8A87
    INY
L8A87:
    TYA
    LDY $0300
    INY
    JSR L8A97
L8A8F:
    DEY
    TYA
    CLC
    ADC #$0A
    JMP L863F
L8A97:
    STX $00
    STY $01
    ASL A
    ASL A
    TAX
    LDY #$20
    LDA $06
    CMP #$D0
    BCC L8AA8
    LDY #$24
L8AA8:
    STY $03
    AND #$0F
    ASL A
    STA $04
    LDA #$00
    STA $05
    LDA $02
    CLC
    ADC #$20
    ASL A
    ROL $05
    ASL A
    ROL $05
    ADC $04
    STA $04
    LDA $05
    ADC #$00
    CLC
    ADC $03
    STA $05
    LDY $01
L8ACD:
    LDA $8A39,X
    STA $0303,Y
    LDA $8A3A,X
    STA $0304,Y
    LDA $8A3B,X
    STA $0308,Y
    LDA $8A3C,X
    STA $0309,Y
    LDA $04
    STA $0301,Y
    CLC
    ADC #$20
    STA $0306,Y
    LDA $05
    STA $0300,Y
    STA $0305,Y
    LDA #$02
    STA $0302,Y
    STA $0307,Y
    LDA #$00
    STA $030A,Y
    LDX $00
    RTS
    .byte $10, $AC, $64, $8C, $8B, $8B, $8C, $8C, $24, $24, $24, $24, $27, $27, $27, $27
    .byte $24, $24, $24, $35, $36, $25, $37, $25, $24, $38, $24, $24, $24, $30, $30, $26
    .byte $26, $26, $34, $26, $24, $31, $24, $32, $33, $26, $24, $33, $34, $26, $26, $26
    .byte $26, $26, $26, $26, $24, $C0, $24, $C0, $24, $7F, $7F, $24, $B8, $BA, $B9, $BB
    .byte $B8, $BC, $B9, $BD, $BA, $BC, $BB, $BD, $60, $64, $61, $65, $62, $66, $63, $67
    .byte $60, $64, $61, $65, $62, $66, $63, $67, $68, $68, $69, $69, $26, $26, $6A, $6A
    .byte $4B, $4C, $4D, $4E, $4D, $4F, $4D, $4F, $4D, $4E, $50, $51, $6B, $70, $2C, $2D
    .byte $6C, $71, $6D, $72, $6E, $73, $6F, $74, $86, $8A, $87, $8B, $88, $8C, $88, $8C
    .byte $89, $8D, $69, $69, $8E, $91, $8F, $92, $26, $93, $26, $93, $90, $94, $69, $69
    .byte $A4, $E9, $EA, $EB, $24, $24, $24, $24, $24, $2F, $24, $3D, $A2, $A2, $A3, $A3
    .byte $24, $24, $24, $24, $A2, $A2, $A3, $A3, $99, $24, $99, $24, $24, $A2, $3E, $3F
    .byte $5B, $5C, $24, $A3, $24, $24, $24, $24, $9D, $47, $9E, $47, $47, $47, $27, $27
    .byte $47, $47, $47, $47, $27, $27, $47, $47, $A9, $47, $AA, $47, $9B, $27, $9C, $27
    .byte $27, $27, $27, $27, $52, $52, $52, $52, $80, $A0, $81, $A1, $BE, $BE, $BF, $BF
    .byte $75, $BA, $76, $BB, $BA, $BA, $BB, $BB, $45, $47, $45, $47, $47, $47, $47, $47
    .byte $45, $47, $45, $47, $B4, $B6, $B5, $B7, $45, $47, $45, $47, $45, $47, $45, $47
    .byte $45, $47, $45, $47, $45, $47, $45, $47, $45, $47, $45, $47, $47, $47, $47, $47
    .byte $47, $47, $47, $47, $47, $47, $47, $47, $47, $47, $47, $47, $47, $47, $47, $47
    .byte $24, $24, $24, $24, $24, $24, $24, $24, $AB, $AC, $AD, $AE, $5D, $5E, $5D, $5E
    .byte $C1, $24, $C1, $24, $C6, $C8, $C7, $C9, $CA, $CC, $CB, $CD, $2A, $2A, $40, $40
    .byte $24, $24, $24, $24, $24, $47, $24, $47, $82, $83, $84, $85, $24, $47, $24, $47
    .byte $86, $8A, $87, $8B, $8E, $91, $8F, $92, $24, $2F, $24, $3D, $24, $24, $24, $35
    .byte $36, $25, $37, $25, $24, $38, $24, $24, $24, $24, $39, $24, $3A, $24, $3B, $24
    .byte $3C, $24, $24, $24, $41, $26, $41, $26, $26, $26, $26, $26, $B0, $B1, $B2, $B3
    .byte $77, $79, $77, $79, $53, $55, $54, $56, $53, $55, $54, $56, $A5, $A7, $A6, $A8
    .byte $C2, $C4, $C3, $C5, $57, $59, $58, $5A, $7B, $7D, $7C, $7E, $3F, $00, $20, $0F
    .byte $15, $12, $25, $0F, $3A, $1A, $0F, $0F, $30, $12, $0F, $0F, $27, $12, $0F, $22
    .byte $16, $27, $18, $0F, $10, $30, $27, $0F, $16, $30, $27, $0F, $0F, $30, $10, $00
    .byte $3F, $00, $20, $0F, $29, $1A, $0F, $0F, $36, $17, $0F, $0F, $30, $21, $0F, $0F
    .byte $27, $17, $0F, $0F, $16, $27, $18, $0F, $1A, $30, $27, $0F, $16, $30, $27, $0F
    .byte $0F, $36, $17, $00, $3F, $00, $20, $0F, $29, $1A, $09, $0F, $3C, $1C, $0F, $0F
    .byte $30, $21, $1C, $0F, $27, $17, $1C, $0F, $16, $27, $18, $0F, $1C, $36, $17, $0F
    .byte $16, $30, $27, $0F, $0C, $3C, $1C, $00, $3F, $00, $20, $0F, $30, $10, $00, $0F
    .byte $30, $10, $00, $0F, $30, $16, $00, $0F, $27, $17, $00, $0F, $16, $27, $18, $0F
    .byte $1C, $36, $17, $0F, $16, $30, $27, $0F, $00, $30, $10, $00, $3F, $00, $04, $22
    .byte $30, $00, $10, $00, $3F, $00, $04, $0F, $30, $00, $10, $00, $3F, $00, $04, $22
    .byte $27, $16, $0F, $00, $3F, $14, $04, $0F, $1A, $30, $27, $00, $25, $48, $10, $1D
    .byte $11, $0A, $17, $14, $24, $22, $18, $1E, $24, $16, $0A, $1B, $12, $18, $2B, $00
    .byte $25, $48, $10, $1D, $11, $0A, $17, $14, $24, $22, $18, $1E, $24, $15, $1E, $12
    .byte $10, $12, $2B, $00, $25, $C5, $16, $0B, $1E, $1D, $24, $18, $1E, $1B, $24, $19
    .byte $1B, $12, $17, $0C, $0E, $1C, $1C, $24, $12, $1C, $24, $12, $17, $26, $05, $0F
    .byte $0A, $17, $18, $1D, $11, $0E, $1B, $24, $0C, $0A, $1C, $1D, $15, $0E, $2B, $00
    .byte $25, $A7, $13, $22, $18, $1E, $1B, $24, $1A, $1E, $0E, $1C, $1D, $24, $12, $1C
    .byte $24, $18, $1F, $0E, $1B, $AF, $00, $25, $E3, $1B, $20, $0E, $24, $19, $1B, $0E
    .byte $1C, $0E, $17, $1D, $24, $22, $18, $1E, $24, $0A, $24, $17, $0E, $20, $24, $1A
    .byte $1E, $0E, $1C, $1D, $AF, $00, $26, $4A, $0D, $19, $1E, $1C, $11, $24, $0B, $1E
    .byte $1D, $1D, $18, $17, $24, $0B, $00, $26, $88, $11, $1D, $18, $24, $1C, $0E, $15
    .byte $0E, $0C, $1D, $24, $0A, $24, $20, $18, $1B, $15, $0D, $00
L8E04:
    ASL A
    TAY
    PLA
    STA $04
    PLA
    STA $05
    INY
    LDA ($04),Y
    STA $06
    INY
    LDA ($04),Y
    STA $07
    JMP ($0006)
L8E19:
    LDA $2002
    LDA $0778
    ORA #$10
    AND #$F0
    JSR L8EED
    LDA #$24
    JSR L8E2D
    LDA #$20
L8E2D:
    STA $2006
    LDA #$00
    STA $2006
    LDX #$04
    LDY #$C0
    LDA #$24
L8E3B:
    STA $2007
    DEY
    BNE L8E3B
    DEX
    BNE L8E3B
    LDY #$40
    TXA
    STA $0300
    STA $0301
L8E4D:
    STA $2007
    DEY
    BNE L8E4D
    STA $073F
    STA $0740
    JMP L8EE6
L8E5C:
    LDA #$01
    STA $4016
    LSR A
    TAX
    STA $4016
    JSR L8E6A
    INX
L8E6A:
    LDY #$08
L8E6C:
    PHA
    LDA $4016,X
    STA $00
    LSR A
    ORA $00
    LSR A
    PLA
    ROL A
    DEY
    BNE L8E6C
    STA $06FC,X
    PHA
    AND #$30
    AND $074A,X
    BEQ L8E8D
    PLA
    AND #$CF
    STA $06FC,X
    RTS
L8E8D:
    PLA
    STA $074A,X
    RTS
L8E92:
    STA $2006
    INY
    LDA ($00),Y
    STA $2006
    INY
    LDA ($00),Y
    ASL A
    PHA
    LDA $0778
    ORA #$04
    BCS L8EA9
    AND #$FB
L8EA9:
    JSR L8EED
    PLA
    ASL A
    BCC L8EB3
    ORA #$02
    INY
L8EB3:
    LSR A
    LSR A
    TAX
L8EB6:
    BCS L8EB9
    INY
L8EB9:
    LDA ($00),Y
    STA $2007
    DEX
    BNE L8EB6
    SEC
    TYA
    ADC $00
    STA $00
    LDA #$00
    ADC $01
    STA $01
    LDA #$3F
    STA $2006
    LDA #$00
    STA $2006
    STA $2006
    STA $2006
L8EDD:
    LDX $2002
    LDY #$00
    LDA ($00),Y
    BNE L8E92
L8EE6:
    STA $2005
    STA $2005
    RTS
L8EED:
    STA $2000
    STA $0778
    RTS
    .byte $F0, $06, $62, $06, $62, $06, $6D, $02, $6D, $02, $7A, $03, $06, $0C, $12, $18
    .byte $1E, $24
L8F06:
    STA $00
    JSR L8F11
    LDA $00
    LSR A
    LSR A
    LSR A
    LSR A
L8F11:
    CLC
    ADC #$01
    AND #$0F
    CMP #$06
    BCS L8F5E
    PHA
    ASL A
    TAY
    LDX $0300
    LDA #$20
    CPY #$00
    BNE L8F28
    LDA #$22
L8F28:
    STA $0301,X
    LDA $8EF4,Y
    STA $0302,X
    LDA $8EF5,Y
    STA $0303,X
    STA $03
    STX $02
    PLA
    TAX
    LDA $8F00,X
    SEC
    SBC $8EF5,Y
    TAY
    LDX $02
L8F47:
    LDA $07D7,Y
    STA $0304,X
    INX
    INY
    DEC $03
    BNE L8F47
    LDA #$00
    STA $0304,X
    INX
    INX
    INX
    STX $0300
L8F5E:
    RTS
L8F5F:
    LDA $0770
    CMP #$00
    BEQ L8F7C
    LDX #$05
L8F68:
    LDA $0134,X
    CLC
    ADC $07D7,Y
    BMI L8F87
    CMP #$0A
    BCS L8F8E
L8F75:
    STA $07D7,Y
    DEY
    DEX
    BPL L8F68
L8F7C:
    LDA #$00
    LDX #$06
L8F80:
    STA $0133,X
    DEX
    BPL L8F80
    RTS
L8F87:
    DEC $0133,X
    LDA #$09
    BNE L8F75
L8F8E:
    SEC
    SBC #$0A
    INC $0133,X
    JMP L8F75
L8F97:
    LDX #$05
    JSR L8F9E
    LDX #$0B
L8F9E:
    LDY #$05
    SEC
L8FA1:
    LDA $07DD,X
    SBC $07D7,Y
    DEX
    DEY
    BPL L8FA1
    BCC L8FBB
    INX
    INY
L8FAF:
    LDA $07DD,X
    STA $07D7,Y
    INX
    INY
    CPY #$06
    BCC L8FAF
L8FBB:
    RTS
    .byte $04, $30, $48, $60, $78, $90, $A8, $C0, $D8, $E8, $24, $F8, $FC, $28, $2C, $18
    .byte $FF, $23, $58
L8FCF:
    LDY #$6F
    JSR L90CC
    LDY #$1F
L8FD6:
    STA $07B0,Y
    DEY
    BPL L8FD6
    LDA #$18
    STA $07A2
    JSR L9C03
L8FE4:
    LDY #$4B
    JSR L90CC
    LDX #$21
    LDA #$00
L8FED:
    STA $0780,X
    DEX
    BPL L8FED
    LDA $075B
    LDY $0752
    BEQ L8FFE
    LDA $0751
L8FFE:
    STA $071A
    STA $0725
    STA $0728
    JSR LB038
    LDY #$20
    AND #$01
    BEQ L9012
    LDY #$24
L9012:
    STY $0720
    LDY #$80
    STY $0721
    ASL A
    ASL A
    ASL A
    ASL A
    STA $06A0
    DEC $0730
    DEC $0731
    DEC $0732
    LDA #$0B
    STA $071E
    JSR L9C22
    LDA $076A
    BNE L9047
    LDA $075F
    CMP #$04
    BCC L904A
    BNE L9047
    LDA $075C
    CMP #$02
    BCC L904A
L9047:
    INC $06CC
L904A:
    LDA $075B
    BEQ L9054
    LDA #$02
    STA $0710
L9054:
    LDA #$80
    STA $FB
    LDA #$01
    STA $0774
    INC $0772
    RTS
L9061:
    LDA #$01
    STA $0757
    STA $0754
    LDA #$02
    STA $075A
    STA $0761
L9071:
    LDA #$00
    STA $0774
    TAY
L9077:
    STA $0300,Y
    INY
    BNE L9077
    STA $0759
    STA $0769
    STA $0728
    LDA #$FF
    STA $03A0
    LDA $071A
    LSR $0778
    AND #$01
    ROR A
    ROL $0778
    JSR L90ED
    LDA #$38
    STA $06E3
    LDA #$48
    STA $06E2
    LDA #$58
    STA $06E1
    LDX #$0E
L90AB:
    LDA $8FBC,X
    STA $06E4,X
    DEX
    BPL L90AB
    LDY #$03
L90B6:
    LDA $8FCB,Y
    STA $0200,Y
    DEY
    BPL L90B6
    JSR L92AF
    JSR L92AA
    INC $0722
    INC $0772
    RTS
L90CC:
    LDX #$07
    LDA #$00
    STA $06
L90D2:
    STX $07
L90D4:
    CPX #$01
    BNE L90DC
    CPY #$60
    BCS L90DE
L90DC:
    STA ($06),Y
L90DE:
    DEY
    CPY #$FF
    BNE L90D4
    DEX
    BPL L90D2
    RTS
    .byte $02, $01, $04, $08, $10, $20
L90ED:
    LDA $0770
    BEQ L9115
    LDA $0752
    CMP #$02
    BEQ L9106
    LDY #$05
    LDA $0710
    CMP #$06
    BEQ L9110
    CMP #$07
    BEQ L9110
L9106:
    LDY $074E
    LDA $0743
    BEQ L9110
    LDY #$04
L9110:
    LDA $90E7,Y
    STA $FB
L9115:
    RTS
    .byte $28, $18, $38, $28, $08, $00, $00, $20, $B0, $50, $00, $00, $B0, $B0, $F0, $00
    .byte $20, $00, $00, $00, $00, $00, $00, $20, $04, $03, $02
L9131:
    LDA $071A
    STA $6D
    LDA #$28
    STA $070A
    LDA #$01
    STA $33
    STA $B5
    LDA #$00
    STA $1D
    DEC $0490
    LDY #$00
    STY $075B
    LDA $074E
    BNE L9153
    INY
L9153:
    STY $0704
    LDX $0710
    LDY $0752
    BEQ L9165
    CPY #$01
    BEQ L9165
    LDX $9118,Y
L9165:
    LDA $9116,Y
    STA $86
    LDA $911C,X
    STA $CE
    LDA $9125,X
    STA $03C4
    JSR L85F1
    LDY $0715
    BEQ L9197
    LDA $0757
    BEQ L9197
    LDA $912D,Y
    STA $07F8
    LDA #$01
    STA $07FA
    LSR A
    STA $07F9
    STA $0757
    STA $079F
L9197:
    LDY $0758
    BEQ L91B0
    LDA #$03
    STA $1D
    LDX #$00
    JSR LBD84
    LDA #$F0
    STA $D7
    LDX #$05
    LDY #$00
    JSR LB91E
L91B0:
    LDY $074E
    BNE L91B8
    JSR LB70B
L91B8:
    LDA #$07
    STA $0E
    RTS
    .byte $56, $40, $65, $70, $66, $40, $66, $40, $66, $40, $66, $60, $65, $70, $00, $00
L91CD:
    INC $0774
    LDA #$00
    STA $0722
    LDA #$80
    STA $FC
    DEC $075A
    BPL L91E9
    LDA #$00
    STA $0772
    LDA #$03
    STA $0770
    RTS
L91E9:
    LDA $075F
    ASL A
    TAX
    LDA $075C
    AND #$02
    BEQ L91F6
    INX
L91F6:
    LDY $91BD,X
    LDA $075C
    LSR A
    TYA
    BCS L9204
    LSR A
    LSR A
    LSR A
    LSR A
L9204:
    AND #$0F
    CMP $071A
    BEQ L920F
    BCC L920F
    LDA #$00
L920F:
    STA $075B
    JSR L9282
    JMP L9264
L9218:
    LDA $0772
    JSR L8E04
    .word L9224
    .word L8567
    .word L9237
L9224:
    LDA #$00
    STA $073C
    STA $0722
    LDA #$02
    STA $FC
    INC $0774
    INC $0772
    RTS
L9237:
    LDA #$00
    STA $0774
    LDA $06FC
    AND #$10
    BNE L9248
    LDA $07A0
    BNE L9281
L9248:
    LDA #$80
    STA $FC
    JSR L9282
    BCC L9264
    LDA $075F
    STA $07FD
    LDA #$00
    ASL A
    STA $0772
    STA $07A0
    STA $0770
    RTS
L9264:
    JSR L9C03
    LDA #$01
    STA $0754
    INC $0757
    LDA #$00
    STA $0747
    STA $0756
    STA $0E
    STA $0772
    LDA #$01
    STA $0770
L9281:
    RTS
L9282:
    SEC
    LDA $077A
    BEQ L92A9
    LDA $0761
    BMI L92A9
    LDA $0753
    EOR #$01
    STA $0753
    LDX #$06
L9297:
    LDA $075A,X
    PHA
    LDA $0761,X
    STA $075A,X
    PLA
    STA $0761,X
    DEX
    BPL L9297
    CLC
L92A9:
    RTS
L92AA:
    LDA #$FF
    STA $06C9
L92AF:
    RTS
L92B0:
    LDY $071F
    BNE L92BA
    LDY #$08
    STY $071F
L92BA:
    DEY
    TYA
    JSR L92C8
    DEC $071F
    BNE L92C7
    JSR L896A
L92C7:
    RTS
L92C8:
    JSR L8E04
    .word L92DB
    .word L88AE
    .word L88AE
    .word L93FC
    .word L92DB
    .word L88AE
    .word L88AE
    .word L93FC
L92DB:
    INC $0726
    LDA $0726
    AND #$0F
    BNE L92EB
    STA $0726
    INC $0725
L92EB:
    INC $06A0
    LDA $06A0
    AND #$1F
    STA $06A0
    RTS
    .byte $00, $30, $60, $93, $00, $00, $11, $12, $12, $13, $00, $00, $51, $52, $53, $00
    .byte $00, $00, $00, $00, $00, $01, $02, $02, $03, $00, $00, $00, $00, $00, $00, $91
    .byte $92, $93, $00, $00, $00, $00, $51, $52, $53, $41, $42, $43, $00, $00, $00, $00
    .byte $00, $91, $92, $97, $87, $88, $89, $99, $00, $00, $00, $11, $12, $13, $A4, $A5
    .byte $A5, $A5, $A6, $97, $98, $99, $01, $02, $03, $00, $A4, $A5, $A6, $00, $11, $12
    .byte $12, $12, $13, $00, $00, $00, $00, $01, $02, $02, $03, $00, $A4, $A5, $A5, $A6
    .byte $00, $00, $00, $11, $12, $12, $13, $00, $00, $00, $00, $00, $00, $00, $9C, $00
    .byte $8B, $AA, $AA, $AA, $AA, $11, $12, $13, $8B, $00, $9C, $9C, $00, $00, $01, $02
    .byte $03, $11, $12, $12, $13, $00, $00, $00, $00, $AA, $AA, $9C, $AA, $00, $8B, $00
    .byte $01, $02, $03, $80, $83, $00, $81, $84, $00, $82, $85, $00, $02, $00, $00, $03
    .byte $00, $00, $04, $00, $00, $00, $05, $06, $07, $06, $0A, $00, $08, $09, $4D, $00
    .byte $00, $0D, $0F, $4E, $0E, $4E, $4E, $00, $0D, $1A, $86, $87, $87, $87, $87, $87
    .byte $87, $87, $87, $87, $87, $69, $69, $00, $00, $00, $00, $00, $45, $47, $47, $47
    .byte $47, $47, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $00, $86
    .byte $87, $69, $54, $52, $62, $00, $00, $00, $18, $01, $18, $07, $18, $0F, $18, $FF
    .byte $18, $01, $1F, $07, $1F, $0F, $1F, $81, $1F, $01, $00, $8F, $1F, $F1, $1F, $F9
    .byte $18, $F1, $18, $FF, $1F
L93FC:
    LDA $0728
    BEQ L9404
    JSR L9508
L9404:
    LDX #$0C
    LDA #$00
L9408:
    STA $06A1,X
    DEX
    BPL L9408
    LDY $0742
    BEQ L9455
    LDA $0725
L9416:
    CMP #$03
    BMI L941F
    SEC
    SBC #$03
    BPL L9416
L941F:
    ASL A
    ASL A
    ASL A
    ASL A
    ADC $92F6,Y
    ADC $0726
    TAX
    LDA $92FA,X
    BEQ L9455
    PHA
    AND #$0F
    SEC
    SBC #$01
    STA $00
    ASL A
    ADC $00
    TAX
    PLA
    LSR A
    LSR A
    LSR A
    LSR A
    TAY
    LDA #$03
    STA $00
L9445:
    LDA $938A,X
    STA $06A1,Y
    INX
    INY
    CPY #$0B
    BEQ L9455
    DEC $00
    BNE L9445
L9455:
    LDX $0741
    BEQ L946D
    LDY $93AD,X
    LDX #$00
L945F:
    LDA $93B1,Y
    BEQ L9467
    STA $06A1,X
L9467:
    INY
    INX
    CPX #$0D
    BNE L945F
L946D:
    LDY $074E
    BNE L947E
    LDA $075F
    CMP #$07
    BNE L947E
    LDA #$62
    JMP L9488
L947E:
    LDA $93D8,Y
    LDY $0743
    BEQ L9488
    LDA #$88
L9488:
    STA $07
    LDX #$00
    LDA $0727
    ASL A
    TAY
L9491:
    LDA $93DC,Y
    STA $00
    INY
    STY $01
    LDA $0743
    BEQ L94A8
    CPX #$00
    BEQ L94A8
    LDA $00
    AND #$08
    STA $00
L94A8:
    LDY #$00
L94AA:
    LDA $C68A,Y
    BIT $00
    BEQ L94B6
    LDA $07
    STA $06A1,X
L94B6:
    INX
    CPX #$0D
    BEQ L94D3
    LDA $074E
    CMP #$02
    BNE L94CA
    CPX #$0B
    BNE L94CA
    LDA #$54
    STA $07
L94CA:
    INY
    CPY #$08
    BNE L94AA
    LDY $01
    BNE L9491
L94D3:
    JSR L9508
    LDA $06A0
    JSR L9BE1
    LDX #$00
    LDY #$00
L94E0:
    STY $00
    LDA $06A1,X
    AND #$C0
    ASL A
    ROL A
    ROL A
    TAY
    LDA $06A1,X
    CMP $9504,Y
    BCS L94F5
    LDA #$00
L94F5:
    LDY $00
    STA ($06),Y
    TYA
    CLC
    ADC #$10
    TAY
    INX
    CPX #$0D
    BCC L94E0
    RTS
    .byte $10, $51, $88, $C0
L9508:
    LDX #$02
L950A:
    STX $08
    LDA #$00
    STA $0729
    LDY $072C
    LDA ($E7),Y
    CMP #$FD
    BEQ L9565
    LDA $0730,X
    BPL L9565
    INY
    LDA ($E7),Y
    ASL A
    BCC L9530
    LDA $072B
    BNE L9530
    INC $072B
    INC $072A
L9530:
    DEY
    LDA ($E7),Y
    AND #$0F
    CMP #$0D
    BNE L9554
    INY
    LDA ($E7),Y
    DEY
    AND #$40
    BNE L955D
    LDA $072B
    BNE L955D
    INY
    LDA ($E7),Y
    AND #$1F
    STA $072A
    INC $072B
    JMP L956E
L9554:
    CMP #$0E
    BNE L955D
    LDA $0728
    BNE L9565
L955D:
    LDA $072A
    CMP $0725
    BCC L956B
L9565:
    JSR L9595
    JMP L9571
L956B:
    INC $0729
L956E:
    JSR L9589
L9571:
    LDX $08
    LDA $0730,X
    BMI L957B
    DEC $0730,X
L957B:
    DEX
    BPL L950A
    LDA $0729
    BNE L9508
    LDA $0728
    BNE L9508
L9588:
    RTS
L9589:
    INC $072C
    INC $072C
    LDA #$00
    STA $072B
    RTS
L9595:
    LDA $0730,X
    BMI L959D
    LDY $072D,X
L959D:
    LDX #$10
    LDA ($E7),Y
    CMP #$FD
    BEQ L9588
    AND #$0F
    CMP #$0F
    BEQ L95B3
    LDX #$08
    CMP #$0C
    BEQ L95B3
    LDX #$00
L95B3:
    STX $07
    LDX $08
    CMP #$0E
    BNE L95C3
    LDA #$00
    STA $07
    LDA #$2E
    BNE L9616
L95C3:
    CMP #$0D
    BNE L95E2
    LDA #$22
    STA $07
    INY
    LDA ($E7),Y
    AND #$40
    BEQ L9635
    LDA ($E7),Y
    AND #$7F
    CMP #$4B
    BNE L95DD
    INC $0745
L95DD:
    AND #$3F
    JMP L9616
L95E2:
    CMP #$0C
    BCS L960D
    INY
    LDA ($E7),Y
    AND #$70
    BNE L95F8
    LDA #$16
    STA $07
    LDA ($E7),Y
    AND #$0F
    JMP L9616
L95F8:
    STA $00
    CMP #$70
    BNE L9608
    LDA ($E7),Y
    AND #$08
    BEQ L9608
    LDA #$00
    STA $00
L9608:
    LDA $00
    JMP L9612
L960D:
    INY
    LDA ($E7),Y
    AND #$70
L9612:
    LSR A
    LSR A
    LSR A
    LSR A
L9616:
    STA $00
    LDA $0730,X
    BPL L965F
    LDA $072A
    CMP $0725
    BEQ L9636
    LDY $072C
    LDA ($E7),Y
    AND #$0F
    CMP #$0E
    BNE L9635
    LDA $0728
    BNE L9656
L9635:
    RTS
L9636:
    LDA $0728
    BEQ L9646
    LDA #$00
    STA $0728
    STA $0729
    STA $08
L9645:
    RTS
L9646:
    LDY $072C
    LDA ($E7),Y
    AND #$F0
    LSR A
    LSR A
    LSR A
    LSR A
    CMP $0726
    BNE L9635
L9656:
    LDA $072C
    STA $072D,X
    JSR L9589
L965F:
    LDA $00
    CLC
    ADC $07
    JSR L8E04
    .word L98E5
    .word L9740
    .word L9A2E
    .word L9A3E
    .word L99F2
    .word L9A50
    .word L9A59
    .word L98E5
    .word L9B41
    .word L97BA
    .word L9979
    .word L997C
    .word L997F
    .word L9957
    .word L9968
    .word L996B
    .word L99D0
    .word L99D7
    .word L9806
    .word L9AB7
    .word L98AB
    .word L9994
    .word L9B0E
    .word L9B0E
    .word L9B0E
    .word L9B01
    .word L9B19
    .word L9B19
    .word L9B19
    .word L9B14
    .word L9B19
    .word L986F
    .word L9A19
    .word L9AD3
    .word L9882
    .word L999E
    .word L9A09
    .word L9A0E
    .word L9A01
    .word L96F2
    .word L970D
    .word L970D
    .word L972B
    .word L972B
    .word L972B
    .word L9645
    .word L96C5
L96C5:
    LDY $072D,X
    INY
    LDA ($E7),Y
    PHA
    AND #$40
    BNE L96E2
    PLA
    PHA
    AND #$0F
    STA $0727
    PLA
    AND #$30
    LSR A
    LSR A
    LSR A
    LSR A
    STA $0742
    RTS
L96E2:
    PLA
    AND #$07
    CMP #$04
    BCC L96EE
    STA $0744
    LDA #$00
L96EE:
    STA $0741
    RTS
L96F2:
    LDX #$04
    LDA $075F
    BEQ L9701
    INX
    LDY $074E
    DEY
    BNE L9701
    INX
L9701:
    TXA
    STA $06D6
    JSR L8808
    LDA #$0D
    JSR L9716
L970D:
    LDA $0723
    EOR #$01
    STA $0723
    RTS
L9716:
    STA $00
    LDA #$00
    LDX #$04
L971C:
    LDY $16,X
    CPY $00
    BNE L9724
    STA $0F,X
L9724:
    DEX
    BPL L971C
    RTS
    .byte $14, $17, $18
L972B:
    LDX $00
    LDA $9720,X
    LDY #$05
L9732:
    DEY
    BMI L973C
    CMP $0016,Y
    BNE L9732
    LDA #$00
L973C:
    STA $06CD
    RTS
L9740:
    LDA $0733
    JSR L8E04
    .word L974C
    .word L9778
    .word L9A69
L974C:
    JSR L9BBB
    LDA $0730,X
    BEQ L9773
    BPL L9767
    TYA
    STA $0730,X
    LDA $0725
    ORA $0726
    BEQ L9767
    LDA #$16
    JMP L97B0
L9767:
    LDX $07
    LDA #$17
    STA $06A1,X
    LDA #$4C
    JMP L97AA
L9773:
    LDA #$18
    JMP L97B0
L9778:
    JSR L9BAC
    STY $06
    BCC L978B
    LDA $0730,X
    LSR A
    STA $0736,X
    LDA #$19
    JMP L97B0
L978B:
    LDA #$1B
    LDY $0730,X
    BEQ L97B0
    LDA $0736,X
    STA $06
    LDX $07
    LDA #$1A
    STA $06A1,X
    CPY $06
    BNE L97CE
    INX
    LDA #$4F
    STA $06A1,X
    LDA #$50
L97AA:
    INX
    LDY #$0F
    JMP L9B7D
L97B0:
    LDX $07
    LDY #$00
    JMP L9B7D
    .byte $42, $41, $43
L97BA:
    JSR L9BAC
    LDY #$00
    BCS L97C8
    INY
    LDA $0730,X
    BNE L97C8
    INY
L97C8:
    LDA $97B7,Y
    STA $06A1
L97CE:
    RTS
    .byte $00, $45, $45, $45, $00, $00, $48, $47, $46, $00, $45, $49, $49, $49, $45, $47
    .byte $47, $4A, $47, $47, $47, $47, $4B, $47, $47, $49, $49, $49, $49, $49, $47, $4A
    .byte $47, $4A, $47, $47, $4B, $47, $4B, $47, $47, $47, $47, $47, $47, $4A, $47, $4A
    .byte $47, $4A, $4B, $47, $4B, $47, $4B
L9806:
    JSR L9BBB
    STY $07
    LDY #$04
    JSR L9BAF
    TXA
    PHA
    LDY $0730,X
    LDX $07
    LDA #$0B
    STA $06
L981B:
    LDA $97CF,Y
    STA $06A1,X
    INX
    LDA $06
    BEQ L982D
    INY
    INY
    INY
    INY
    INY
    DEC $06
L982D:
    CPX #$0B
    BNE L981B
    PLA
    TAX
    LDA $0725
    BEQ L986E
    LDA $0730,X
    CMP #$01
    BEQ L9869
    LDY $07
    BNE L9847
    CMP #$03
    BEQ L9869
L9847:
    CMP #$02
    BNE L986E
    JSR L9BCB
    PHA
    JSR L994A
    PLA
    STA $87,X
    LDA $0725
    STA $6E,X
    LDA #$01
    STA $B6,X
    STA $0F,X
    LDA #$90
    STA $CF,X
    LDA #$31
    STA $16,X
    RTS
L9869:
    LDY #$52
    STY $06AB
L986E:
    RTS
L986F:
    JSR L9BBB
    LDY $0730,X
    LDX $07
    LDA #$6B
    STA $06A1,X
    LDA #$6C
    STA $06A2,X
    RTS
L9882:
    LDY #$03
    JSR L9BAF
    LDY #$0A
    JSR L98B3
    BCS L989E
    LDX #$06
L9890:
    LDA #$00
    STA $06A1,X
    DEX
    BPL L9890
    LDA $98DD,Y
    STA $06A8
L989E:
    RTS
    .byte $15, $14, $00, $00, $15, $1E, $1D, $1C, $15, $21, $20, $1F
L98AB:
    LDY #$03
    JSR L9BAF
    JSR L9BBB
L98B3:
    DEY
    DEY
    STY $05
    LDY $0730,X
    STY $06
    LDX $05
    INX
    LDA $989F,Y
    CMP #$00
    BEQ L98CE
    LDX #$00
    LDY $05
    JSR L9B7D
    CLC
L98CE:
    LDY $06
    LDA $98A3,Y
    STA $06A1,X
    LDA $98A7,Y
    STA $06A2,X
    RTS
    .byte $11, $10, $15, $14, $13, $12, $15, $14
L98E5:
    JSR L9939
    LDA $00
    BEQ L98F0
    INY
    INY
    INY
    INY
L98F0:
    TYA
    PHA
    LDA $0760
    ORA $075F
    BEQ L9925
    LDY $0730,X
    BEQ L9925
    JSR L994A
    BCS L9925
    JSR L9BCB
    CLC
    ADC #$08
    STA $87,X
    LDA $0725
    ADC #$00
    STA $6E,X
    LDA #$01
    STA $B6,X
    STA $0F,X
    JSR L9BD3
    STA $CF,X
    LDA #$0D
    STA $16,X
    JSR LC787
L9925:
    PLA
    TAY
    LDX $07
    LDA $98DD,Y
    STA $06A1,X
    INX
    LDA $98DF,Y
    LDY $06
    DEY
    JMP L9B7D
L9939:
    LDY #$01
    JSR L9BAF
    JSR L9BBB
    TYA
    AND #$07
    STA $06
    LDY $0730,X
    RTS
L994A:
    LDX #$00
L994C:
    CLC
    LDA $0F,X
    BEQ L9956
    INX
    CPX #$05
    BNE L994C
L9956:
    RTS
L9957:
    JSR L9BAC
    LDA #$86
    STA $06AB
    LDX #$0B
    LDY #$01
    LDA #$87
    JMP L9B7D
L9968:
    LDA #$03
    .byte $2C
L996B:
    LDA #$07
    PHA
    JSR L9BAC
    PLA
    TAX
    LDA #$C0
    STA $06A1,X
    RTS
L9979:
    LDA #$06
    .byte $2C
L997C:
    LDA #$07
    .byte $2C
L997F:
    LDA #$09
    PHA
    JSR L9BAC
    PLA
    TAX
    LDA #$0B
    STA $06A1,X
    INX
    LDY #$00
    LDA #$63
    JMP L9B7D
L9994:
    JSR L9BBB
    LDX #$02
    LDA #$6D
    JMP L9B7D
L999E:
    LDA #$24
    STA $06A1
    LDX #$01
    LDY #$08
    LDA #$25
    JSR L9B7D
    LDA #$61
    STA $06AB
    JSR L9BCB
    SEC
    SBC #$08
    STA $8C
    LDA $0725
    SBC #$00
    STA $73
    LDA #$30
    STA $D4
    LDA #$B0
    STA $010D
    LDA #$30
    STA $1B
    INC $14
    RTS
L99D0:
    LDX #$00
    LDY #$0F
    JMP L99E9
L99D7:
    TXA
    PHA
    LDX #$01
    LDY #$0F
    LDA #$44
    JSR L9B7D
    PLA
    TAX
    JSR L9BBB
    LDX #$01
L99E9:
    LDA #$40
    JMP L9B7D
    .byte $C3, $C2, $C2, $C2
L99F2:
    LDY $074E
    LDA $99EE,Y
    JMP L9A44
    .byte $06, $07, $08, $C5, $0C, $89
L9A01:
    LDY #$0C
    JSR L9BAF
    JMP L9A0E
L9A09:
    LDA #$08
    STA $0773
L9A0E:
    LDY $00
    LDX $99F9,Y
    LDA $99FC,Y
    JMP L9A20
L9A19:
    JSR L9BBB
    LDX $07
    LDA #$C4
L9A20:
    LDY #$00
    JMP L9B7D
    .byte $69, $61, $61, $62, $22, $51, $52, $52, $88
L9A2E:
    LDY $074E
    LDA $0743
    BEQ L9A38
    LDY #$04
L9A38:
    LDA $9A29,Y
    JMP L9A44
L9A3E:
    LDY $074E
    LDA $9A25,Y
L9A44:
    PHA
    JSR L9BAC
L9A48:
    LDX $07
    LDY #$00
    PLA
    JMP L9B7D
L9A50:
    LDY $074E
    LDA $9A29,Y
    JMP L9A5F
L9A59:
    LDY $074E
    LDA $9A25,Y
L9A5F:
    PHA
    JSR L9BBB
    PLA
    LDX $07
    JMP L9B7D
L9A69:
    JSR L9BBB
    LDX $07
    LDA #$64
    STA $06A1,X
    INX
    DEY
    BMI L9A85
    LDA #$65
    STA $06A1,X
    INX
    DEY
    BMI L9A85
    LDA #$66
    JSR L9B7D
L9A85:
    LDX $046A
    JSR L9BD3
    STA $0477,X
    LDA $0725
    STA $046B,X
    JSR L9BCB
    STA $0471,X
    INX
    CPX #$06
    BCC L9AA1
    LDX #$00
L9AA1:
    STX $046A
    RTS
    .byte $07, $07, $06, $05, $04, $03, $02, $01, $00, $03, $03, $04, $05, $06, $07, $08
    .byte $09, $0A
L9AB7:
    JSR L9BAC
    BCC L9AC1
    LDA #$09
    STA $0734
L9AC1:
    DEC $0734
    LDY $0734
    LDX $9AAE,Y
    LDA $9AA5,Y
    TAY
    LDA #$61
    JMP L9B7D
L9AD3:
    JSR L9BBB
    JSR L994A
    JSR L9BCB
    STA $87,X
    LDA $0725
    STA $6E,X
    JSR L9BD3
    STA $CF,X
    STA $58,X
    LDA #$32
    STA $16,X
    LDY #$01
    STY $B6,X
    INC $0F,X
    LDX $07
    LDA #$67
    STA $06A1,X
    LDA #$68
    STA $06A2,X
    RTS
L9B01:
    LDA $075D
    BEQ L9B3C
    LDA #$00
    STA $075D
    JMP L9B19
L9B0E:
    JSR L9B36
    JMP L9B2C
L9B14:
    LDA #$00
    STA $06BC
L9B19:
    JSR L9B36
    STY $07
    LDA #$00
    LDY $074E
    DEY
    BEQ L9B28
    LDA #$05
L9B28:
    CLC
    ADC $07
    TAY
L9B2C:
    LDA $BDE8,Y
    PHA
    JSR L9BBB
    JMP L9A48
L9B36:
    LDA $00
    SEC
    SBC #$00
    TAY
L9B3C:
    RTS
    .byte $87, $00, $00, $00
L9B41:
    JSR L9BAC
    BCC L9B73
    LDA $074E
    BNE L9B73
    LDX $046A
    JSR L9BCB
    SEC
    SBC #$10
    STA $0471,X
    LDA $0725
    SBC #$00
    STA $046B,X
    INY
    INY
    TYA
    ASL A
    ASL A
    ASL A
    ASL A
    STA $0477,X
    INX
    CPX #$05
    BCC L9B70
    LDX #$00
L9B70:
    STX $046A
L9B73:
    LDX $074E
    LDA $9B3D,X
    LDX #$08
    LDY #$0F
L9B7D:
    STY $0735
    LDY $06A1,X
    BEQ L9B9D
    CPY #$17
    BEQ L9BA0
    CPY #$1A
    BEQ L9BA0
    CPY #$C0
    BEQ L9B9D
    CPY #$C0
    BCS L9BA0
    CPY #$54
    BNE L9B9D
    CMP #$50
    BEQ L9BA0
L9B9D:
    STA $06A1,X
L9BA0:
    INX
    CPX #$0D
    BCS L9BAB
    LDY $0735
    DEY
    BPL L9B7D
L9BAB:
    RTS
L9BAC:
    JSR L9BBB
L9BAF:
    LDA $0730,X
    CLC
    BPL L9BBA
    TYA
    STA $0730,X
    SEC
L9BBA:
    RTS
L9BBB:
    LDY $072D,X
    LDA ($E7),Y
    AND #$0F
    STA $07
    INY
    LDA ($E7),Y
    AND #$0F
    TAY
    RTS
L9BCB:
    LDA $0726
    ASL A
    ASL A
    ASL A
    ASL A
    RTS
L9BD3:
    LDA $07
    ASL A
    ASL A
    ASL A
    ASL A
    CLC
    ADC #$20
    RTS
    .byte $00, $D0, $05, $05
L9BE1:
    PHA
    LSR A
    LSR A
    LSR A
    LSR A
    TAY
    LDA $9BDF,Y
    STA $07
    PLA
    AND #$0F
    CLC
    ADC $9BDD,Y
    STA $06
    RTS
    .byte $FF, $FF, $12, $36, $0E, $0E, $0E, $32, $32, $32, $0A, $26, $40
L9C03:
    JSR L9C13
    STA $0750
L9C09:
    AND #$60
    ASL A
    ROL A
    ROL A
    ROL A
    STA $074E
    RTS
L9C13:
    LDY $075F
    LDA $9CB4,Y
    CLC
    ADC $0760
    TAY
    LDA $9CBC,Y
    RTS
L9C22:
    LDA $0750
    JSR L9C09
    TAY
    LDA $0750
    AND #$1F
    STA $074F
    LDA $9CE0,Y
    CLC
    ADC $074F
    TAY
    LDA $9CE4,Y
    STA $E9
    LDA $9D06,Y
    STA $EA
    LDY $074E
    LDA $9D28,Y
    CLC
    ADC $074F
    TAY
    LDA $9D2C,Y
    STA $E7
    LDA $9D4E,Y
    STA $E8
    LDY #$00
    LDA ($E7),Y
    PHA
    AND #$07
    CMP #$04
    BCC L9C68
    STA $0744
    LDA #$00
L9C68:
    STA $0741
    PLA
    PHA
    AND #$38
    LSR A
    LSR A
    LSR A
    STA $0710
    PLA
    AND #$C0
    CLC
    ROL A
    ROL A
    ROL A
    STA $0715
    INY
    LDA ($E7),Y
    PHA
    AND #$0F
    STA $0727
    PLA
    PHA
    AND #$30
    LSR A
    LSR A
    LSR A
    LSR A
    STA $0742
    PLA
    AND #$C0
    CLC
    ROL A
    ROL A
    ROL A
    CMP #$03
    BNE L9CA3
    STA $0743
    LDA #$00
L9CA3:
    STA $0733
    LDA $E7
    CLC
    ADC #$02
    STA $E7
    LDA $E8
    ADC #$00
    STA $E8
    RTS
    .byte $00, $05, $0A, $0E, $13, $17, $1B, $20, $25, $29, $C0, $26, $60, $28, $29, $01
    .byte $27, $62, $24, $35, $20, $63, $22, $29, $41, $2C, $61, $2A, $31, $26, $62, $2E
    .byte $23, $2D, $60, $33, $29, $01, $27, $64, $30, $32, $21, $65, $1F, $06, $1C, $00
    .byte $70, $97, $B0, $DF, $0A, $1F, $59, $7E, $9B, $A9, $D0, $01, $1F, $3C, $51, $7B
    .byte $7C, $A0, $A9, $CE, $F1, $FA, $FB, $35, $60, $8E, $AA, $B3, $D8, $05, $33, $60
    .byte $71, $9B, $9D, $9D, $9D, $9D, $9E, $9E, $9E, $9E, $9E, $9E, $9E, $9F, $9F, $9F
    .byte $9F, $9F, $9F, $9F, $9F, $9F, $9F, $9F, $9F, $A0, $A0, $A0, $A0, $A0, $A0, $A1
    .byte $A1, $A1, $A1, $A1, $00, $03, $19, $1C, $06, $45, $C0, $6B, $CE, $37, $8A, $19
    .byte $8E, $F3, $48, $CD, $32, $3B, $7A, $8F, $F6, $5B, $CE, $FF, $92, $05, $7E, $D7
    .byte $02, $35, $D8, $79, $AF, $10, $8F, $02, $6F, $FA, $AE, $AE, $AE, $A4, $A4, $A5
    .byte $A5, $A6, $A6, $A6, $A7, $A7, $A8, $A8, $A8, $A8, $A8, $A9, $A9, $A9, $AA, $AB
    .byte $AB, $AB, $AC, $AC, $AC, $AD, $A1, $A2, $A2, $A3, $A3, $A3, $76, $DD, $BB, $4C
    .byte $EA, $1D, $1B, $CC, $56, $5D, $16, $9D, $C6, $1D, $36, $9D, $C9, $1D, $04, $DB
    .byte $49, $1D, $84, $1B, $C9, $5D, $88, $95, $0F, $08, $30, $4C, $78, $2D, $A6, $28
    .byte $90, $B5, $FF, $0F, $03, $56, $1B, $C9, $1B, $0F, $07, $36, $1B, $AA, $1B, $48
    .byte $95, $0F, $0A, $2A, $1B, $5B, $0C, $78, $2D, $90, $B5, $FF, $0B, $8C, $4B, $4C
    .byte $77, $5F, $EB, $0C, $BD, $DB, $19, $9D, $75, $1D, $7D, $5B, $D9, $1D, $3D, $DD
    .byte $99, $1D, $26, $9D, $5A, $2B, $8A, $2C, $CA, $1B, $20, $95, $7B, $5C, $DB, $4C
    .byte $1B, $CC, $3B, $CC, $78, $2D, $A6, $28, $90, $B5, $FF, $0B, $8C, $3B, $1D, $8B
    .byte $1D, $AB, $0C, $DB, $1D, $0F, $03, $65, $1D, $6B, $1B, $05, $9D, $0B, $1B, $05
    .byte $9B, $0B, $1D, $8B, $0C, $1B, $8C, $70, $15, $7B, $0C, $DB, $0C, $0F, $08, $78
    .byte $2D, $A6, $28, $90, $B5, $FF, $27, $A9, $4B, $0C, $68, $29, $0F, $06, $77, $1B
    .byte $0F, $0B, $60, $15, $4B, $8C, $78, $2D, $90, $B5, $FF, $0F, $03, $8E, $65, $E1
    .byte $BB, $38, $6D, $A8, $3E, $E5, $E7, $0F, $08, $0B, $02, $2B, $02, $5E, $65, $E1
    .byte $BB, $0E, $DB, $0E, $BB, $8E, $DB, $0E, $FE, $65, $EC, $0F, $0D, $4E, $65, $E1
    .byte $0F, $0E, $4E, $02, $E0, $0F, $10, $FE, $E5, $E1, $1B, $85, $7B, $0C, $5B, $95
    .byte $78, $2D, $90, $B5, $FF, $A5, $86, $E4, $28, $18, $A8, $45, $83, $69, $03, $C6
    .byte $29, $9B, $83, $16, $A4, $88, $24, $E9, $28, $05, $A8, $7B, $28, $24, $8F, $C8
    .byte $03, $E8, $03, $46, $A8, $85, $24, $C8, $24, $FF, $EB, $8E, $0F, $03, $FB, $05
    .byte $17, $85, $DB, $8E, $0F, $07, $57, $05, $7B, $05, $9B, $80, $2B, $85, $FB, $05
    .byte $0F, $0B, $1B, $05, $9B, $05, $FF, $2E, $C2, $66, $E2, $11, $0F, $07, $02, $11
    .byte $0F, $0C, $12, $11, $FF, $0E, $C2, $A8, $AB, $00, $BB, $8E, $6B, $82, $DE, $00
    .byte $A0, $33, $86, $43, $06, $3E, $B4, $A0, $CB, $02, $0F, $07, $7E, $42, $A6, $83
    .byte $02, $0F, $0A, $3B, $02, $CB, $37, $0F, $0C, $E3, $0E, $FF, $9B, $8E, $CA, $0E
    .byte $EE, $42, $44, $5B, $86, $80, $B8, $1B, $80, $50, $BA, $10, $B7, $5B, $00, $17
    .byte $85, $4B, $05, $FE, $34, $40, $B7, $86, $C6, $06, $5B, $80, $83, $00, $D0, $38
    .byte $5B, $8E, $8A, $0E, $A6, $00, $BB, $0E, $C5, $80, $F3, $00, $FF, $1E, $C2, $00
    .byte $6B, $06, $8B, $86, $63, $B7, $0F, $05, $03, $06, $23, $06, $4B, $B7, $BB, $00
    .byte $5B, $B7, $FB, $37, $3B, $B7, $0F, $0B, $1B, $37, $FF, $2B, $D7, $E3, $03, $C2
    .byte $86, $E2, $06, $76, $A5, $A3, $8F, $03, $86, $2B, $57, $68, $28, $E9, $28, $E5
    .byte $83, $24, $8F, $36, $A8, $5B, $03, $FF, $0F, $02, $78, $40, $48, $CE, $F8, $C3
    .byte $F8, $C3, $0F, $07, $7B, $43, $C6, $D0, $0F, $8A, $C8, $50, $FF, $85, $86, $0B
    .byte $80, $1B, $00, $DB, $37, $77, $80, $EB, $37, $FE, $2B, $20, $2B, $80, $7B, $38
    .byte $AB, $B8, $77, $86, $FE, $42, $20, $49, $86, $8B, $06, $9B, $80, $7B, $8E, $5B
    .byte $B7, $9B, $0E, $BB, $0E, $9B, $80, $FF, $0B, $80, $60, $38, $10, $B8, $C0, $3B
    .byte $DB, $8E, $40, $B8, $F0, $38, $7B, $8E, $A0, $B8, $C0, $B8, $FB, $00, $A0, $B8
    .byte $30, $BB, $EE, $42, $88, $0F, $0B, $2B, $0E, $67, $0E, $FF, $0A, $AA, $0E, $28
    .byte $2A, $0E, $31, $88, $FF, $C7, $83, $D7, $03, $42, $8F, $7A, $03, $05, $A4, $78
    .byte $24, $A6, $25, $E4, $25, $4B, $83, $E3, $03, $05, $A4, $89, $24, $B5, $24, $09
    .byte $A4, $65, $24, $C9, $24, $0F, $08, $85, $25, $FF, $CD, $A5, $B5, $A8, $07, $A8
    .byte $76, $28, $CC, $25, $65, $A4, $A9, $24, $E5, $24, $19, $A4, $0F, $07, $95, $28
    .byte $E6, $24, $19, $A4, $D7, $29, $16, $A9, $58, $29, $97, $29, $FF, $0F, $02, $02
    .byte $11, $0F, $07, $02, $11, $FF, $FF, $2B, $82, $AB, $38, $DE, $42, $E2, $1B, $B8
    .byte $EB, $3B, $DB, $80, $8B, $B8, $1B, $82, $FB, $B8, $7B, $80, $FB, $3C, $5B, $BC
    .byte $7B, $B8, $1B, $8E, $CB, $0E, $1B, $8E, $0F, $0D, $2B, $3B, $BB, $B8, $EB, $82
    .byte $4B, $B8, $BB, $38, $3B, $B7, $BB, $02, $0F, $13, $1B, $00, $CB, $80, $6B, $BC
    .byte $FF, $7B, $80, $AE, $00, $80, $8B, $8E, $E8, $05, $F9, $86, $17, $86, $16, $85
    .byte $4E, $2B, $80, $AB, $8E, $87, $85, $C3, $05, $8B, $82, $9B, $02, $AB, $02, $BB
    .byte $86, $CB, $06, $D3, $03, $3B, $8E, $6B, $0E, $A7, $8E, $FF, $29, $8E, $52, $11
    .byte $83, $0E, $0F, $03, $9B, $0E, $2B, $8E, $5B, $0E, $CB, $8E, $FB, $0E, $FB, $82
    .byte $9B, $82, $BB, $02, $FE, $42, $E8, $BB, $8E, $0F, $0A, $AB, $0E, $CB, $0E, $F9
    .byte $0E, $88, $86, $A6, $06, $DB, $02, $B6, $8E, $FF, $AB, $CE, $DE, $42, $C0, $CB
    .byte $CE, $5B, $8E, $1B, $CE, $4B, $85, $67, $45, $0F, $07, $2B, $00, $7B, $85, $97
    .byte $05, $0F, $0A, $92, $02, $FF, $0A, $AA, $0E, $24, $4A, $1E, $23, $AA, $FF, $1B
    .byte $80, $BB, $38, $4B, $BC, $EB, $3B, $0F, $04, $2B, $00, $AB, $38, $EB, $00, $CB
    .byte $8E, $FB, $80, $AB, $B8, $6B, $80, $FB, $3C, $9B, $BB, $5B, $BC, $FB, $00, $6B
    .byte $B8, $FB, $38, $FF, $0B, $86, $1A, $06, $DB, $06, $DE, $C2, $02, $F0, $3B, $BB
    .byte $80, $EB, $06, $0B, $86, $93, $06, $F0, $39, $0F, $06, $60, $B8, $1B, $86, $A0
    .byte $B9, $B7, $27, $BD, $27, $2B, $83, $A1, $26, $A9, $26, $EE, $25, $0B, $27, $B4
    .byte $FF, $0F, $02, $1E, $2F, $60, $E0, $3A, $A5, $A7, $DB, $80, $3B, $82, $8B, $02
    .byte $FE, $42, $68, $70, $BB, $25, $A7, $2C, $27, $B2, $26, $B9, $26, $9B, $80, $A8
    .byte $82, $B5, $27, $BC, $27, $B0, $BB, $3B, $82, $87, $34, $EE, $25, $6B, $FF, $1E
    .byte $A5, $0A, $2E, $28, $27, $2E, $33, $C7, $0F, $03, $1E, $40, $07, $2E, $30, $E7
    .byte $0F, $05, $1E, $24, $44, $0F, $07, $1E, $22, $6A, $2E, $23, $AB, $0F, $09, $1E
    .byte $41, $68, $1E, $2A, $8A, $2E, $23, $A2, $2E, $32, $EA, $FF, $3B, $87, $66, $27
    .byte $CC, $27, $EE, $31, $87, $EE, $23, $A7, $3B, $87, $DB, $07, $FF, $0F, $01, $2E
    .byte $25, $2B, $2E, $25, $4B, $4E, $25, $CB, $6B, $07, $97, $47, $E9, $87, $47, $C7
    .byte $7A, $07, $D6, $C7, $78, $07, $38, $87, $AB, $47, $E3, $07, $9B, $87, $0F, $09
    .byte $68, $47, $DB, $C7, $3B, $C7, $FF, $47, $9B, $CB, $07, $FA, $1D, $86, $9B, $3A
    .byte $87, $56, $07, $88, $1B, $07, $9D, $2E, $65, $F0, $FF, $9B, $07, $05, $32, $06
    .byte $33, $07, $34, $CE, $03, $DC, $51, $EE, $07, $73, $E0, $74, $0A, $7E, $06, $9E
    .byte $0A, $CE, $06, $E4, $00, $E8, $0A, $FE, $0A, $2E, $89, $4E, $0B, $54, $0A, $14
    .byte $8A, $C4, $0A, $34, $8A, $7E, $06, $C7, $0A, $01, $E0, $02, $0A, $47, $0A, $81
    .byte $60, $82, $0A, $C7, $0A, $0E, $87, $7E, $02, $A7, $02, $B3, $02, $D7, $02, $E3
    .byte $02, $07, $82, $13, $02, $3E, $06, $7E, $02, $AE, $07, $FE, $0A, $0D, $C4, $CD
    .byte $43, $CE, $09, $DE, $0B, $DD, $42, $FE, $02, $5D, $C7, $FD, $5B, $07, $05, $32
    .byte $06, $33, $07, $34, $5E, $0A, $68, $64, $98, $64, $A8, $64, $CE, $06, $FE, $02
    .byte $0D, $01, $1E, $0E, $7E, $02, $94, $63, $B4, $63, $D4, $63, $F4, $63, $14, $E3
    .byte $2E, $0E, $5E, $02, $64, $35, $88, $72, $BE, $0E, $0D, $04, $AE, $02, $CE, $08
    .byte $CD, $4B, $FE, $02, $0D, $05, $68, $31, $7E, $0A, $96, $31, $A9, $63, $A8, $33
    .byte $D5, $30, $EE, $02, $E6, $62, $F4, $61, $04, $B1, $08, $3F, $44, $33, $94, $63
    .byte $A4, $31, $E4, $31, $04, $BF, $08, $3F, $04, $BF, $08, $3F, $CD, $4B, $03, $E4
    .byte $0E, $03, $2E, $01, $7E, $06, $BE, $02, $DE, $06, $FE, $0A, $0D, $C4, $CD, $43
    .byte $CE, $09, $DE, $0B, $DD, $42, $FE, $02, $5D, $C7, $FD, $9B, $07, $05, $32, $06
    .byte $33, $07, $34, $FE, $00, $27, $B1, $65, $32, $75, $0A, $71, $00, $B7, $31, $08
    .byte $E4, $18, $64, $1E, $04, $57, $3B, $BB, $0A, $17, $8A, $27, $3A, $73, $0A, $7B
    .byte $0A, $D7, $0A, $E7, $3A, $3B, $8A, $97, $0A, $FE, $08, $24, $8A, $2E, $00, $3E
    .byte $40, $38, $64, $6F, $00, $9F, $00, $BE, $43, $C8, $0A, $C9, $63, $CE, $07, $FE
    .byte $07, $2E, $81, $66, $42, $6A, $42, $79, $0A, $BE, $00, $C8, $64, $F8, $64, $08
    .byte $E4, $2E, $07, $7E, $03, $9E, $07, $BE, $03, $DE, $07, $FE, $0A, $03, $A5, $0D
    .byte $44, $CD, $43, $CE, $09, $DD, $42, $DE, $0B, $FE, $02, $5D, $C7, $FD, $9B, $07
    .byte $05, $32, $06, $33, $07, $34, $FE, $06, $0C, $81, $39, $0A, $5C, $01, $89, $0A
    .byte $AC, $01, $D9, $0A, $FC, $01, $2E, $83, $A7, $01, $B7, $00, $C7, $01, $DE, $0A
    .byte $FE, $02, $4E, $83, $5A, $32, $63, $0A, $69, $0A, $7E, $02, $EE, $03, $FA, $32
    .byte $03, $8A, $09, $0A, $1E, $02, $EE, $03, $FA, $32, $03, $8A, $09, $0A, $14, $42
    .byte $1E, $02, $7E, $0A, $9E, $07, $FE, $0A, $2E, $86, $5E, $0A, $8E, $06, $BE, $0A
    .byte $EE, $07, $3E, $83, $5E, $07, $FE, $0A, $0D, $C4, $41, $52, $51, $52, $CD, $43
    .byte $CE, $09, $DE, $0B, $DD, $42, $FE, $02, $5D, $C7, $FD, $5B, $07, $05, $32, $06
    .byte $33, $07, $34, $FE, $0A, $AE, $86, $BE, $07, $FE, $02, $0D, $02, $27, $32, $46
    .byte $61, $55, $62, $5E, $0E, $1E, $82, $68, $3C, $74, $3A, $7D, $4B, $5E, $8E, $7D
    .byte $4B, $7E, $82, $84, $62, $94, $61, $A4, $31, $BD, $4B, $CE, $06, $FE, $02, $0D
    .byte $06, $34, $31, $3E, $0A, $64, $32, $75, $0A, $7B, $61, $A4, $33, $AE, $02, $DE
    .byte $0E, $3E, $82, $64, $32, $78, $32, $B4, $36, $C8, $36, $DD, $4B, $44, $B2, $58
    .byte $32, $94, $63, $A4, $3E, $BA, $30, $C9, $61, $CE, $06, $DD, $4B, $CE, $86, $DD
    .byte $4B, $FE, $02, $2E, $86, $5E, $02, $7E, $06, $FE, $02, $1E, $86, $3E, $02, $5E
    .byte $06, $7E, $02, $9E, $06, $FE, $0A, $0D, $C4, $CD, $43, $CE, $09, $DE, $0B, $DD
    .byte $42, $FE, $02, $5D, $C7, $FD, $5B, $06, $05, $32, $06, $33, $07, $34, $5E, $0A
    .byte $AE, $02, $0D, $01, $39, $73, $0D, $03, $39, $7B, $4D, $4B, $DE, $06, $1E, $8A
    .byte $AE, $06, $C4, $33, $16, $FE, $A5, $77, $FE, $02, $FE, $82, $0D, $07, $39, $73
    .byte $A8, $74, $ED, $4B, $49, $FB, $E8, $74, $FE, $0A, $2E, $82, $67, $02, $84, $7A
    .byte $87, $31, $0D, $0B, $FE, $02, $0D, $0C, $39, $73, $5E, $06, $C6, $76, $45, $FF
    .byte $BE, $0A, $DD, $48, $FE, $06, $3D, $CB, $46, $7E, $AD, $4A, $FE, $82, $39, $F3
    .byte $A9, $7B, $4E, $8A, $9E, $07, $FE, $0A, $0D, $C4, $CD, $43, $CE, $09, $DE, $0B
    .byte $DD, $42, $FE, $02, $5D, $C7, $FD, $94, $11, $0F, $26, $FE, $10, $28, $94, $65
    .byte $15, $EB, $12, $FA, $41, $4A, $96, $54, $40, $A4, $42, $B7, $13, $E9, $19, $F5
    .byte $15, $11, $80, $47, $42, $71, $13, $80, $41, $15, $92, $1B, $1F, $24, $40, $55
    .byte $12, $64, $40, $95, $12, $A4, $40, $D2, $12, $E1, $40, $13, $C0, $2C, $17, $2F
    .byte $12, $49, $13, $83, $40, $9F, $14, $A3, $40, $17, $92, $83, $13, $92, $41, $B9
    .byte $14, $C5, $12, $C8, $40, $D4, $40, $4B, $92, $78, $1B, $9C, $94, $9F, $11, $DF
    .byte $14, $FE, $11, $7D, $C1, $9E, $42, $CF, $20, $FD, $90, $B1, $0F, $26, $29, $91
    .byte $7E, $42, $FE, $40, $28, $92, $4E, $42, $2E, $C0, $57, $73, $C3, $25, $C7, $27
    .byte $23, $84, $33, $20, $5C, $01, $77, $63, $88, $62, $99, $61, $AA, $60, $BC, $01
    .byte $EE, $42, $4E, $C0, $69, $11, $7E, $42, $DE, $40, $F8, $62, $0E, $C2, $AE, $40
    .byte $D7, $63, $E7, $63, $33, $A7, $37, $27, $43, $04, $CC, $01, $E7, $73, $0C, $81
    .byte $3E, $42, $0D, $0A, $5E, $40, $88, $72, $BE, $42, $E7, $87, $FE, $40, $39, $E1
    .byte $4E, $00, $69, $60, $87, $60, $A5, $60, $C3, $31, $FE, $31, $6D, $C1, $BE, $42
    .byte $EF, $20, $FD, $52, $21, $0F, $20, $6E, $40, $58, $F2, $93, $01, $97, $00, $0C
    .byte $81, $97, $40, $A6, $41, $C7, $40, $0D, $04, $03, $01, $07, $01, $23, $01, $27
    .byte $01, $EC, $03, $AC, $F3, $C3, $03, $78, $E2, $94, $43, $47, $F3, $74, $43, $47
    .byte $FB, $74, $43, $2C, $F1, $4C, $63, $47, $00, $57, $21, $5C, $01, $7C, $72, $39
    .byte $F1, $EC, $02, $4C, $81, $D8, $62, $EC, $01, $0D, $0D, $0F, $38, $C7, $07, $ED
    .byte $4A, $1D, $C1, $5F, $26, $FD, $54, $21, $0F, $26, $A7, $22, $37, $FB, $73, $20
    .byte $83, $07, $87, $02, $93, $20, $C7, $73, $04, $F1, $06, $31, $39, $71, $59, $71
    .byte $E7, $73, $37, $A0, $47, $04, $86, $7C, $E5, $71, $E7, $31, $33, $A4, $39, $71
    .byte $A9, $71, $D3, $23, $08, $F2, $13, $05, $27, $02, $49, $71, $75, $75, $E8, $72
    .byte $67, $F3, $99, $71, $E7, $20, $F4, $72, $F7, $31, $17, $A0, $33, $20, $39, $71
    .byte $73, $28, $BC, $05, $39, $F1, $79, $71, $A6, $21, $C3, $06, $D3, $20, $DC, $00
    .byte $FC, $00, $07, $A2, $13, $21, $5F, $32, $8C, $00, $98, $7A, $C7, $63, $D9, $61
    .byte $03, $A2, $07, $22, $74, $72, $77, $31, $E7, $73, $39, $F1, $58, $72, $77, $73
    .byte $D8, $72, $7F, $B1, $97, $73, $B6, $64, $C5, $65, $D4, $66, $E3, $67, $F3, $67
    .byte $8D, $C1, $CF, $26, $FD, $52, $31, $0F, $20, $6E, $66, $07, $81, $36, $01, $66
    .byte $00, $A7, $22, $08, $F2, $67, $7B, $DC, $02, $98, $F2, $D7, $20, $39, $F1, $9F
    .byte $33, $DC, $27, $DC, $57, $23, $83, $57, $63, $6C, $51, $87, $63, $99, $61, $A3
    .byte $06, $B3, $21, $77, $F3, $F3, $21, $F7, $2A, $13, $81, $23, $22, $53, $00, $63
    .byte $22, $E9, $0B, $0C, $83, $13, $21, $16, $22, $33, $05, $8F, $35, $EC, $01, $63
    .byte $A0, $67, $20, $73, $01, $77, $01, $83, $20, $87, $20, $B3, $20, $B7, $20, $C3
    .byte $01, $C7, $00, $D3, $20, $D7, $20, $67, $A0, $77, $07, $87, $22, $E8, $62, $F5
    .byte $65, $1C, $82, $7F, $38, $8D, $C1, $CF, $26, $FD, $50, $21, $07, $81, $47, $24
    .byte $57, $00, $63, $01, $77, $01, $C9, $71, $68, $F2, $E7, $73, $97, $FB, $06, $83
    .byte $5C, $01, $D7, $22, $E7, $00, $03, $A7, $6C, $02, $B3, $22, $E3, $01, $E7, $07
    .byte $47, $A0, $57, $06, $A7, $01, $D3, $00, $D7, $01, $07, $81, $67, $20, $93, $22
    .byte $03, $A3, $1C, $61, $17, $21, $6F, $33, $C7, $63, $D8, $62, $E9, $61, $FA, $60
    .byte $4F, $B3, $87, $63, $9C, $01, $B7, $63, $C8, $62, $D9, $61, $EA, $60, $39, $F1
    .byte $87, $21, $A7, $01, $B7, $20, $39, $F1, $5F, $38, $6D, $C1, $AF, $26, $FD, $90
    .byte $11, $0F, $26, $FE, $10, $2A, $93, $87, $17, $A3, $14, $B2, $42, $0A, $92, $19
    .byte $40, $36, $14, $50, $41, $82, $16, $2B, $93, $24, $41, $BB, $14, $B8, $00, $C2
    .byte $43, $C3, $13, $1B, $94, $67, $12, $C4, $15, $53, $C1, $D2, $41, $12, $C1, $29
    .byte $13, $85, $17, $1B, $92, $1A, $42, $47, $13, $83, $41, $A7, $13, $0E, $91, $A7
    .byte $63, $B7, $63, $C5, $65, $D5, $65, $DD, $4A, $E3, $67, $F3, $67, $8D, $C1, $AE
    .byte $42, $DF, $20, $FD, $90, $11, $0F, $26, $6E, $10, $8B, $17, $AF, $32, $D8, $62
    .byte $E8, $62, $FC, $3F, $AD, $C8, $F8, $64, $0C, $BE, $43, $43, $F8, $64, $0C, $BF
    .byte $73, $40, $84, $40, $93, $40, $A4, $40, $B3, $40, $F8, $64, $48, $E4, $5C, $39
    .byte $83, $40, $92, $41, $B3, $40, $F8, $64, $48, $E4, $5C, $39, $F8, $64, $13, $C2
    .byte $37, $65, $4C, $24, $63, $00, $97, $65, $C3, $42, $0B, $97, $AC, $32, $F8, $64
    .byte $0C, $BE, $53, $45, $9D, $48, $F8, $64, $2A, $E2, $3C, $47, $56, $43, $BA, $62
    .byte $F8, $64, $0C, $B7, $88, $64, $BC, $31, $D4, $45, $FC, $31, $3C, $B1, $78, $64
    .byte $8C, $38, $0B, $9C, $1A, $33, $18, $61, $28, $61, $39, $60, $5D, $4A, $EE, $11
    .byte $0F, $B8, $1D, $C1, $3E, $42, $6F, $20, $FD, $52, $31, $0F, $20, $6E, $40, $F7
    .byte $20, $07, $84, $17, $20, $4F, $34, $C3, $03, $C7, $02, $D3, $22, $27, $E3, $39
    .byte $61, $E7, $73, $5C, $E4, $57, $00, $6C, $73, $47, $A0, $53, $06, $63, $22, $A7
    .byte $73, $FC, $73, $13, $A1, $33, $05, $43, $21, $5C, $72, $C3, $23, $CC, $03, $77
    .byte $FB, $AC, $02, $39, $F1, $A7, $73, $D3, $04, $E8, $72, $E3, $22, $26, $F4, $BC
    .byte $02, $8C, $81, $A8, $62, $17, $87, $43, $24, $A7, $01, $C3, $04, $08, $F2, $97
    .byte $21, $A3, $02, $C9, $0B, $E1, $69, $F1, $69, $8D, $C1, $CF, $26, $FD, $38, $11
    .byte $0F, $26, $AD, $40, $3D, $C7, $FD, $95, $B1, $0F, $26, $0D, $02, $C8, $72, $1C
    .byte $81, $38, $72, $0D, $05, $97, $34, $98, $62, $A3, $20, $B3, $06, $C3, $20, $CC
    .byte $03, $F9, $91, $2C, $81, $48, $62, $0D, $09, $37, $63, $47, $03, $57, $21, $8C
    .byte $02, $C5, $79, $C7, $31, $F9, $11, $39, $F1, $A9, $11, $6F, $B4, $D3, $65, $E3
    .byte $65, $7D, $C1, $BF, $26, $FD, $00, $C1, $4C, $00, $F4, $4F, $0D, $02, $02, $42
    .byte $43, $4F, $52, $C2, $DE, $00, $5A, $C2, $4D, $C7, $FD, $90, $51, $0F, $26, $EE
    .byte $10, $0B, $94, $33, $14, $42, $42, $77, $16, $86, $44, $02, $92, $4A, $16, $69
    .byte $42, $73, $14, $B0, $00, $C7, $12, $05, $C0, $1C, $17, $1F, $11, $36, $12, $8F
    .byte $14, $91, $40, $1B, $94, $35, $12, $34, $42, $60, $42, $61, $12, $87, $12, $96
    .byte $40, $A3, $14, $1C, $98, $1F, $11, $47, $12, $9F, $15, $CC, $15, $CF, $11, $05
    .byte $C0, $1F, $15, $39, $12, $7C, $16, $7F, $11, $82, $40, $98, $12, $DF, $15, $16
    .byte $C4, $17, $14, $54, $12, $9B, $16, $28, $94, $CE, $01, $3D, $C1, $5E, $42, $8F
    .byte $20, $FD, $97, $11, $0F, $26, $FE, $10, $2B, $92, $57, $12, $8B, $12, $C0, $41
    .byte $F7, $13, $5B, $92, $69, $0B, $BB, $12, $B2, $46, $19, $93, $71, $00, $17, $94
    .byte $7C, $14, $7F, $11, $93, $41, $BF, $15, $FC, $13, $FF, $11, $2F, $95, $50, $42
    .byte $51, $12, $58, $14, $A6, $12, $DB, $12, $1B, $93, $46, $43, $7B, $12, $8D, $49
    .byte $B7, $14, $1B, $94, $49, $0B, $BB, $12, $FC, $13, $FF, $12, $03, $C1, $2F, $15
    .byte $43, $12, $4B, $13, $77, $13, $9D, $4A, $15, $C1, $A1, $41, $C3, $12, $FE, $01
    .byte $7D, $C1, $9E, $42, $CF, $20, $FD, $52, $21, $0F, $20, $6E, $44, $0C, $F1, $4C
    .byte $01, $AA, $35, $D9, $34, $EE, $20, $08, $B3, $37, $32, $43, $04, $4E, $21, $53
    .byte $20, $7C, $01, $97, $21, $B7, $07, $9C, $81, $E7, $42, $5F, $B3, $97, $63, $AC
    .byte $02, $C5, $41, $49, $E0, $58, $61, $76, $64, $85, $65, $94, $66, $A4, $22, $A6
    .byte $03, $C8, $22, $DC, $02, $68, $F2, $96, $42, $13, $82, $17, $02, $AF, $34, $F6
    .byte $21, $FC, $06, $26, $80, $2A, $24, $36, $01, $8C, $00, $FF, $35, $4E, $A0, $55
    .byte $21, $77, $20, $87, $07, $89, $22, $AE, $21, $4C, $82, $9F, $34, $EC, $01, $03
    .byte $E7, $13, $67, $8D, $4A, $AD, $41, $0F, $A6, $FD, $10, $51, $4C, $00, $C7, $12
    .byte $C6, $42, $03, $92, $02, $42, $29, $12, $63, $12, $62, $42, $69, $14, $A5, $12
    .byte $A4, $42, $E2, $14, $E1, $44, $F8, $16, $37, $C1, $8F, $38, $02, $BB, $28, $7A
    .byte $68, $7A, $A8, $7A, $E0, $6A, $F0, $6A, $6D, $C5, $FD, $92, $31, $0F, $20, $6E
    .byte $40, $0D, $02, $37, $73, $EC, $00, $0C, $80, $3C, $00, $6C, $00, $9C, $00, $06
    .byte $C0, $C7, $73, $06, $83, $28, $72, $96, $40, $E7, $73, $26, $C0, $87, $7B, $D2
    .byte $41, $39, $F1, $C8, $F2, $97, $E3, $A3, $23, $E7, $02, $E3, $07, $F3, $22, $37
    .byte $E3, $9C, $00, $BC, $00, $EC, $00, $0C, $80, $3C, $00, $86, $21, $A6, $06, $B6
    .byte $24, $5C, $80, $7C, $00, $9C, $00, $29, $E1, $DC, $05, $F6, $41, $DC, $80, $E8
    .byte $72, $0C, $81, $27, $73, $4C, $01, $66, $74, $0D, $11, $3F, $35, $B6, $41, $2C
    .byte $82, $36, $40, $7C, $02, $86, $40, $F9, $61, $39, $E1, $AC, $04, $C6, $41, $0C
    .byte $83, $16, $41, $88, $F2, $39, $F1, $7C, $00, $89, $61, $9C, $00, $A7, $63, $BC
    .byte $00, $C5, $65, $DC, $00, $E3, $67, $F3, $67, $8D, $C1, $CF, $26, $FD, $55, $B1
    .byte $0F, $26, $CF, $33, $07, $B2, $15, $11, $52, $42, $99, $0B, $AC, $02, $D3, $24
    .byte $D6, $42, $D7, $25, $23, $84, $CF, $33, $07, $E3, $19, $61, $78, $7A, $EF, $33
    .byte $2C, $81, $46, $64, $55, $65, $65, $65, $EC, $74, $47, $82, $53, $05, $63, $21
    .byte $62, $41, $96, $22, $9A, $41, $CC, $03, $B9, $91, $39, $F1, $63, $26, $67, $27
    .byte $D3, $06, $FC, $01, $18, $E2, $D9, $07, $E9, $04, $0C, $86, $37, $22, $93, $24
    .byte $87, $84, $AC, $02, $C2, $41, $C3, $23, $D9, $71, $FC, $01, $7F, $B1, $9C, $00
    .byte $A7, $63, $B6, $64, $CC, $00, $D4, $66, $E3, $67, $F3, $67, $8D, $C1, $CF, $26
    .byte $FD, $50, $B1, $0F, $26, $FC, $00, $1F, $B3, $5C, $00, $65, $65, $74, $66, $83
    .byte $67, $93, $67, $DC, $73, $4C, $80, $B3, $20, $C9, $0B, $C3, $08, $D3, $2F, $DC
    .byte $00, $2C, $80, $4C, $00, $8C, $00, $D3, $2E, $ED, $4A, $FC, $00, $D7, $A1, $EC
    .byte $01, $4C, $80, $59, $11, $D8, $11, $DA, $10, $37, $A0, $47, $04, $99, $11, $E7
    .byte $21, $3A, $90, $67, $20, $76, $10, $77, $60, $87, $07, $D8, $12, $39, $F1, $AC
    .byte $00, $E9, $71, $0C, $80, $2C, $00, $4C, $05, $C7, $7B, $39, $F1, $EC, $00, $F9
    .byte $11, $0C, $82, $6F, $34, $F8, $11, $FA, $10, $7F, $B2, $AC, $00, $B6, $64, $CC
    .byte $01, $E3, $67, $F3, $67, $8D, $C1, $CF, $26, $FD, $52, $B1, $0F, $20, $6E, $45
    .byte $39, $91, $B3, $04, $C3, $21, $C8, $11, $CA, $10, $49, $91, $7C, $73, $E8, $12
    .byte $88, $91, $8A, $10, $E7, $21, $05, $91, $07, $30, $17, $07, $27, $20, $49, $11
    .byte $9C, $01, $C8, $72, $23, $A6, $27, $26, $D3, $03, $D8, $7A, $89, $91, $D8, $72
    .byte $39, $F1, $A9, $11, $09, $F1, $63, $24, $67, $24, $D8, $62, $28, $91, $2A, $10
    .byte $56, $21, $70, $04, $79, $0B, $8C, $00, $94, $21, $9F, $35, $2F, $B8, $3D, $C1
    .byte $7F, $26, $FD, $06, $C1, $4C, $00, $F4, $4F, $0D, $02, $06, $20, $24, $4F, $35
    .byte $A0, $36, $20, $53, $46, $D5, $20, $D6, $20, $34, $A1, $73, $49, $74, $20, $94
    .byte $20, $B4, $20, $D4, $20, $F4, $20, $2E, $80, $59, $42, $4D, $C7, $FD, $96, $31
    .byte $0F, $26, $0D, $03, $1A, $60, $77, $42, $C4, $00, $C8, $62, $B9, $E1, $D3, $06
    .byte $D7, $07, $F9, $61, $0C, $81, $4E, $B1, $8E, $B1, $BC, $01, $E4, $50, $E9, $61
    .byte $0C, $81, $0D, $0A, $84, $43, $98, $72, $0D, $0C, $0F, $38, $1D, $C1, $5F, $26
    .byte $FD, $48, $0F, $0E, $01, $5E, $02, $A7, $00, $BC, $73, $1A, $E0, $39, $61, $58
    .byte $62, $77, $63, $97, $63, $B8, $62, $D6, $07, $F8, $62, $19, $E1, $75, $52, $86
    .byte $40, $87, $50, $95, $52, $93, $43, $A5, $21, $C5, $52, $D6, $40, $D7, $20, $E5
    .byte $06, $E6, $51, $3E, $8D, $5E, $03, $67, $52, $77, $52, $7E, $02, $9E, $03, $A6
    .byte $43, $A7, $23, $DE, $05, $FE, $02, $1E, $83, $33, $54, $46, $40, $47, $21, $56
    .byte $04, $5E, $02, $83, $54, $93, $52, $96, $07, $97, $50, $BE, $03, $C7, $23, $FE
    .byte $02, $0C, $82, $43, $45, $45, $24, $46, $24, $90, $08, $95, $51, $78, $FA, $D7
    .byte $73, $39, $F1, $8C, $01, $A8, $52, $B8, $52, $CC, $01, $5F, $B3, $97, $63, $9E
    .byte $00, $0E, $81, $16, $24, $66, $04, $8E, $00, $FE, $01, $08, $D2, $0E, $06, $6F
    .byte $47, $9E, $0F, $0E, $82, $2D, $47, $28, $7A, $68, $7A, $A8, $7A, $AE, $01, $DE
    .byte $0F, $6D, $C5, $FD, $48, $0F, $0E, $01, $5E, $02, $BC, $01, $FC, $01, $2C, $82
    .byte $41, $52, $4E, $04, $67, $25, $68, $24, $69, $24, $BA, $42, $C7, $04, $DE, $0B
    .byte $B2, $87, $FE, $02, $2C, $E1, $2C, $71, $67, $01, $77, $00, $87, $01, $8E, $00
    .byte $EE, $01, $F6, $02, $03, $85, $05, $02, $13, $21, $16, $02, $27, $02, $2E, $02
    .byte $88, $72, $C7, $20, $D7, $07, $E4, $76, $07, $A0, $17, $06, $48, $7A, $76, $20
    .byte $98, $72, $79, $E1, $88, $62, $9C, $01, $B7, $73, $DC, $01, $F8, $62, $FE, $01
    .byte $08, $E2, $0E, $00, $6E, $02, $73, $20, $77, $23, $83, $04, $93, $20, $AE, $00
    .byte $FE, $0A, $0E, $82, $39, $71, $A8, $72, $E7, $73, $0C, $81, $8F, $32, $AE, $00
    .byte $FE, $04, $04, $D1, $17, $04, $26, $49, $27, $29, $DF, $33, $FE, $02, $44, $F6
    .byte $7C, $01, $8E, $06, $BF, $47, $EE, $0F, $4D, $C7, $0E, $82, $68, $7A, $AE, $01
    .byte $DE, $0F, $6D, $C5, $FD, $48, $01, $0E, $01, $00, $5A, $3E, $06, $45, $46, $47
    .byte $46, $53, $44, $AE, $01, $DF, $4A, $4D, $C7, $0E, $81, $00, $5A, $2E, $04, $37
    .byte $28, $3A, $48, $46, $47, $C7, $07, $CE, $0F, $DF, $4A, $4D, $C7, $0E, $81, $00
    .byte $5A, $33, $53, $43, $51, $46, $40, $47, $50, $53, $04, $55, $40, $56, $50, $62
    .byte $43, $64, $40, $65, $50, $71, $41, $73, $51, $83, $51, $94, $40, $95, $50, $A3
    .byte $50, $A5, $40, $A6, $50, $B3, $51, $B6, $40, $B7, $50, $C3, $53, $DF, $4A, $4D
    .byte $C7, $0E, $81, $00, $5A, $2E, $02, $36, $47, $37, $52, $3A, $49, $47, $25, $A7
    .byte $52, $D7, $04, $DF, $4A, $4D, $C7, $0E, $81, $00, $5A, $3E, $02, $44, $51, $53
    .byte $44, $54, $44, $55, $24, $A1, $54, $AE, $01, $B4, $21, $DF, $4A, $E5, $07, $4D
    .byte $C7, $FD, $41, $01, $B4, $34, $C8, $52, $F2, $51, $47, $D3, $6C, $03, $65, $49
    .byte $9E, $07, $BE, $01, $CC, $03, $FE, $07, $0D, $C9, $1E, $01, $6C, $01, $62, $35
    .byte $63, $53, $8A, $41, $AC, $01, $B3, $53, $E9, $51, $26, $C3, $27, $33, $63, $43
    .byte $64, $33, $BA, $60, $C9, $61, $CE, $0B, $E5, $09, $EE, $0F, $7D, $CA, $7D, $47
    .byte $FD, $41, $01, $B8, $52, $EA, $41, $27, $B2, $B3, $42, $16, $D4, $4A, $42, $A5
    .byte $51, $A7, $31, $27, $D3, $08, $E2, $16, $64, $2C, $04, $38, $42, $76, $64, $88
    .byte $62, $DE, $07, $FE, $01, $0D, $C9, $23, $32, $31, $51, $98, $52, $0D, $C9, $59
    .byte $42, $63, $53, $67, $31, $14, $C2, $36, $31, $87, $53, $17, $E3, $29, $61, $30
    .byte $62, $3C, $08, $42, $37, $59, $40, $6A, $42, $99, $40, $C9, $61, $D7, $63, $39
    .byte $D1, $58, $52, $C3, $67, $D3, $31, $DC, $06, $F7, $42, $FA, $42, $23, $B1, $43
    .byte $67, $C3, $34, $C7, $34, $D1, $51, $43, $B3, $47, $33, $9A, $30, $A9, $61, $B8
    .byte $62, $BE, $0B, $D5, $09, $DE, $0F, $0D, $CA, $7D, $47, $FD, $49, $0F, $1E, $01
    .byte $39, $73, $5E, $07, $AE, $0B, $1E, $82, $6E, $88, $9E, $02, $0D, $04, $2E, $0B
    .byte $45, $09, $4E, $0F, $ED, $47, $FD, $FF
LAEDC:
    LDA $0772
    JSR L8E04
    .word L8FE4
    .word L8567
    .word L9071
    .word LAEEA
LAEEA:
    LDX $0753
    LDA $06FC,X
    STA $06FC
    JSR LB04A
    LDA $0772
    CMP #$03
    BCS LAEFE
    RTS
LAEFE:
    JSR LB624
    LDX #$00
LAF03:
    STX $08
    JSR LC047
    JSR L84C3
    INX
    CPX #$06
    BNE LAF03
    JSR LF180
    JSR LF12A
    JSR LEEE9
    JSR LBED4
    LDX #$01
    STX $08
    JSR LBE70
    DEX
    STX $08
    JSR LBE70
    JSR LBB96
    JSR LB9BC
    JSR LB7B8
    JSR LB855
    JSR LB74F
    JSR L89E1
    LDA $B5
    CMP #$02
    BPL LAF52
    LDA $079F
    BEQ LAF64
    CMP #$04
    BNE LAF52
    LDA $077F
    BNE LAF52
    JSR L90ED
LAF52:
    LDY $079F
    LDA $09
    CPY #$08
    BCS LAF5D
    LSR A
    LSR A
LAF5D:
    LSR A
    JSR LB288
    JMP LAF67
LAF64:
    JSR LB29A
LAF67:
    LDA $0A
    STA $0D
    LDA #$00
    STA $0C
LAF6F:
    LDA $0773
    CMP #$06
    BEQ LAF92
    LDA $071F
    BNE LAF8F
    LDA $073D
    CMP #$20
    BMI LAF92
    LDA $073D
    SBC #$20
    STA $073D
    LDA #$00
    STA $0340
LAF8F:
    JSR L92B0
LAF92:
    RTS
LAF93:
    LDA $06FF
    CLC
    ADC $03A1
    STA $06FF
    LDA $0723
    BNE LAFFB
    LDA $0755
    CMP #$50
    BCC LAFFB
    LDA $0785
    BNE LAFFB
    LDY $06FF
    DEY
    BMI LAFFB
    INY
    CPY #$02
    BCC LAFBA
    DEY
LAFBA:
    LDA $0755
    CMP #$70
    BCC LAFC4
    LDY $06FF
LAFC4:
    TYA
    STA $0775
    CLC
    ADC $073D
    STA $073D
    TYA
    CLC
    ADC $071C
    STA $071C
    STA $073F
    LDA $071A
    ADC #$00
    STA $071A
    AND #$01
    STA $00
    LDA $0778
    AND #$FE
    ORA $00
    STA $0778
    JSR LB038
    LDA #$08
    STA $0795
    JMP LB000
LAFFB:
    LDA #$00
    STA $0775
LB000:
    LDX #$00
    JSR LF1F6
    STA $00
    LDY #$00
    ASL A
    BCS LB013
    INY
    LDA $00
    AND #$20
    BEQ LB02E
LB013:
    LDA $071C,Y
    SEC
    SBC $B034,Y
    STA $86
    LDA $071A,Y
    SBC #$00
    STA $6D
    LDA $0C
    CMP $B036,Y
    BEQ LB02E
    LDA #$00
    STA $57
LB02E:
    LDA #$00
    STA $03A1
    RTS
    .byte $00, $10, $01, $02
LB038:
    LDA $071C
    CLC
    ADC #$FF
    STA $071D
    LDA $071A
    ADC #$00
    STA $071B
    RTS
LB04A:
    LDA $0E
    JSR L8E04
    .word L9131
    .word LB1C7
    .word LB206
    .word LB1E5
    .word LB2A4
    .word LB2CA
    .word L91CD
    .word LB069
    .word LB0E9
    .word LB233
    .word LB245
    .word LB269
    .word LB27D
LB069:
    LDA $0752
    CMP #$02
    BEQ LB09B
    LDA #$00
    LDY $CE
    CPY #$30
    BCC LB0E6
    LDA $0710
    CMP #$06
    BEQ LB083
    CMP #$07
    BNE LB0D3
LB083:
    LDA $03C4
    BNE LB08D
    LDA #$01
    JMP LB0E6
LB08D:
    JSR LB21F
    DEC $06DE
    BNE LB0E5
    INC $0769
    JMP LB315
LB09B:
    LDA $0758
    BNE LB0AC
    LDA #$FF
    JSR LB200
    LDA $CE
    CMP #$91
    BCC LB0D3
    RTS
LB0AC:
    LDA $0399
    CMP #$60
    BNE LB0E5
    LDA $CE
    CMP #$99
    LDY #$00
    LDA #$01
    BCC LB0C7
    LDA #$03
    STA $1D
    INY
    LDA #$08
    STA $05B4
LB0C7:
    STY $0716
    JSR LB0E6
    LDA $86
    CMP #$48
    BCC LB0E5
LB0D3:
    LDA #$08
    STA $0E
    LDA #$01
    STA $33
    LSR A
    STA $0752
    STA $0716
    STA $0758
LB0E5:
    RTS
LB0E6:
    STA $06FC
LB0E9:
    LDA $0E
    CMP #$0B
    BEQ LB12B
    LDA $074E
    BNE LB104
    LDY $B5
    DEY
    BNE LB0FF
    LDA $CE
    CMP #$D0
    BCC LB104
LB0FF:
    LDA #$00
    STA $06FC
LB104:
    LDA $06FC
    AND #$C0
    STA $0A
    LDA $06FC
    AND #$03
    STA $0C
    LDA $06FC
    AND #$0C
    STA $0B
    AND #$04
    BEQ LB12B
    LDA $1D
    BNE LB12B
    LDY $0C
    BEQ LB12B
    LDA #$00
    STA $0C
    STA $0B
LB12B:
    JSR LB329
    LDY #$01
    LDA $0754
    BNE LB13E
    LDY #$00
    LDA $0714
    BEQ LB13E
    LDY #$02
LB13E:
    STY $0499
    LDA #$01
    LDY $57
    BEQ LB14C
    BPL LB14A
    ASL A
LB14A:
    STA $45
LB14C:
    JSR LAF93
    JSR LF180
    JSR LF12A
    LDX #$00
    JSR LE29C
    JSR LDC64
    LDA $CE
    CMP #$40
    BCC LB179
    LDA $0E
    CMP #$05
    BEQ LB179
    CMP #$07
    BEQ LB179
    CMP #$04
    BCC LB179
    LDA $03C4
    AND #$DF
    STA $03C4
LB179:
    LDA $B5
    CMP #$02
    BMI LB1BA
    LDX #$01
    STX $0723
    LDY #$04
    STY $07
    LDX #$00
    LDY $0759
    BNE LB194
    LDY $0743
    BNE LB1AA
LB194:
    INX
    LDY $0E
    CPY #$0B
    BEQ LB1AA
    LDY $0712
    BNE LB1A6
    INY
    STY $FC
    STY $0712
LB1A6:
    LDY #$06
    STY $07
LB1AA:
    CMP $07
    BMI LB1BA
    DEX
    BMI LB1BB
    LDY $07B1
    BNE LB1BA
    LDA #$06
    STA $0E
LB1BA:
    RTS
LB1BB:
    LDA #$00
    STA $0758
    JSR LB1DD
    INC $0752
    RTS
LB1C7:
    LDA $B5
    BNE LB1D1
    LDA $CE
    CMP #$E4
    BCC LB1DD
LB1D1:
    LDA #$08
    STA $0758
    LDY #$03
    STY $1D
    JMP LB0E6
LB1DD:
    LDA #$02
    STA $0752
    JMP LB213
LB1E5:
    LDA #$01
    JSR LB200
    JSR LAF93
    LDY #$00
    LDA $06D6
    BNE LB20B
    INY
    LDA $074E
    CMP #$03
    BNE LB20B
    INY
    JMP LB20B
LB200:
    CLC
    ADC $CE
    STA $CE
    RTS
LB206:
    JSR LB21F
    LDY #$02
LB20B:
    DEC $06DE
    BNE LB21E
    STY $0752
LB213:
    INC $0774
    LDA #$00
    STA $0772
    STA $0722
LB21E:
    RTS
LB21F:
    LDA #$08
    STA $57
    LDY #$01
    LDA $86
    AND #$0F
    BNE LB22E
    STA $57
    TAY
LB22E:
    TYA
    JSR LB0E6
    RTS
LB233:
    LDA $0747
    CMP #$F8
    BNE LB23D
    JMP LB255
LB23D:
    CMP #$C4
    BNE LB244
    JSR LB273
LB244:
    RTS
LB245:
    LDA $0747
    CMP #$F0
    BCS LB253
    CMP #$C8
    BEQ LB273
    JMP LB0E9
LB253:
    BNE LB268
LB255:
    LDY $070B
    BNE LB268
    STY $070D
    INC $070B
    LDA $0754
    EOR #$01
    STA $0754
LB268:
    RTS
LB269:
    LDA $0747
    CMP #$F0
    BCS LB2A3
    JMP LB0E9
LB273:
    LDA #$00
    STA $0747
    LDA #$08
    STA $0E
    RTS
LB27D:
    LDA $0747
    CMP #$C0
    BEQ LB297
    LDA $09
    LSR A
    LSR A
LB288:
    AND #$03
    STA $00
    LDA $03C4
    AND #$FC
    ORA $00
    STA $03C4
    RTS
LB297:
    JSR LB273
LB29A:
    LDA $03C4
    AND #$FC
    STA $03C4
    RTS
LB2A3:
    RTS
LB2A4:
    LDA $1B
    CMP #$30
    BNE LB2BF
    LDA $0713
    STA $FF
    LDA #$00
    STA $0713
    LDY $CE
    CPY #$9E
    BCS LB2BC
    LDA #$04
LB2BC:
    JMP LB0E6
LB2BF:
    INC $0E
    RTS
    .byte $15, $23, $16, $1B, $17, $18, $23, $63
LB2CA:
    LDA #$01
    JSR LB0E6
    LDA $CE
    CMP #$AE
    BCC LB2E3
    LDA $0723
    BEQ LB2E3
    LDA #$20
    STA $FC
    LDA #$00
    STA $0723
LB2E3:
    LDA $0490
    LSR A
    BCS LB2F6
    LDA $0746
    BNE LB2F1
    INC $0746
LB2F1:
    LDA #$20
    STA $03C4
LB2F6:
    LDA $0746
    CMP #$05
    BNE LB328
    INC $075C
    LDA $075C
    CMP #$03
    BNE LB315
    LDY $075F
    LDA $0748
    CMP $B2C2,Y
    BCC LB315
    INC $075D
LB315:
    INC $0760
    JSR L9C03
    INC $0757
    JSR LB213
    STA $075B
    LDA #$80
    STA $FC
LB328:
    RTS
LB329:
    LDA #$00
    LDY $0754
    BNE LB338
    LDA $1D
    BNE LB33B
    LDA $0B
    AND #$04
LB338:
    STA $0714
LB33B:
    JSR LB450
    LDA $070B
    BNE LB359
    LDA $1D
    CMP #$03
    BEQ LB34E
    LDY #$18
    STY $0789
LB34E:
    JSR L8E04
    .word LB35A
    .word LB376
    .word LB36D
    .word LB3CF
LB359:
    RTS
LB35A:
    JSR LB58F
    LDA $0C
    BEQ LB363
    STA $33
LB363:
    JSR LB5CC
    JSR LBF09
    STA $06FF
    RTS
LB36D:
    LDA $070A
    STA $0709
    JMP LB3AC
LB376:
    LDY $9F
    BPL LB38D
    LDA $0A
    AND #$80
    AND $0D
    BNE LB393
    LDA $0708
    SEC
    SBC $CE
    CMP $0706
    BCC LB393
LB38D:
    LDA $070A
    STA $0709
LB393:
    LDA $0704
    BEQ LB3AC
    JSR LB58F
    LDA $CE
    CMP #$14
    BCS LB3A6
    LDA #$18
    STA $0709
LB3A6:
    LDA $0C
    BEQ LB3AC
    STA $33
LB3AC:
    LDA $0C
    BEQ LB3B3
    JSR LB5CC
LB3B3:
    JSR LBF09
    STA $06FF
    LDA $0E
    CMP #$0B
    BNE LB3C4
    LDA #$28
    STA $0709
LB3C4:
    JMP LBF4D
    .byte $0E, $04, $FC, $F2, $00, $00, $FF, $FF
LB3CF:
    LDA $0416
    CLC
    ADC $0433
    STA $0416
    LDY #$00
    LDA $9F
    BPL LB3E0
    DEY
LB3E0:
    STY $00
    ADC $CE
    STA $CE
    LDA $B5
    ADC $00
    STA $B5
    LDA $0C
    AND $0490
    BEQ LB420
    LDY $0789
    BNE LB41F
    LDY #$18
    STY $0789
    LDX #$00
    LDY $33
    LSR A
    BCS LB406
    INX
    INX
LB406:
    DEY
    BEQ LB40A
    INX
LB40A:
    LDA $86
    CLC
    ADC $B3C7,X
    STA $86
    LDA $6D
    ADC $B3CB,X
    STA $6D
    LDA $0C
    EOR #$03
    STA $33
LB41F:
    RTS
LB420:
    STA $0789
    RTS
    .byte $20, $20, $1E, $28, $28, $0D, $04, $70, $70, $60, $90, $90, $0A, $09, $FC, $FC
    .byte $FC, $FB, $FB, $FE, $FF, $00, $00, $00, $00, $00, $80, $00, $D8, $E8, $F0, $28
    .byte $18, $10, $0C, $E4, $98, $D0, $00, $FF, $01, $00, $20, $FF
LB450:
    LDA $1D
    CMP #$03
    BNE LB479
    LDY #$00
    LDA $0B
    AND $0490
    BEQ LB465
    INY
    AND #$08
    BNE LB465
    INY
LB465:
    LDX $B44D,Y
    STX $0433
    LDA #$08
    LDX $B44A,Y
    STX $9F
    BMI LB475
    LSR A
LB475:
    STA $070C
    RTS
LB479:
    LDA $070E
    BNE LB488
    LDA $0A
    AND #$80
    BEQ LB488
    AND $0D
    BEQ LB48B
LB488:
    JMP LB51C
LB48B:
    LDA $1D
    BEQ LB4A0
    LDA $0704
    BEQ LB488
    LDA $0782
    BNE LB4A0
    LDA $9F
    BPL LB4A0
    JMP LB51C
LB4A0:
    LDA #$20
    STA $0782
    LDY #$00
    STY $0416
    STY $0433
    LDA $B5
    STA $0707
    LDA $CE
    STA $0708
    LDA #$01
    STA $1D
    LDA $0700
    CMP #$09
    BCC LB4D2
    INY
    CMP #$10
    BCC LB4D2
    INY
    CMP #$19
    BCC LB4D2
    INY
    CMP #$1C
    BCC LB4D2
    INY
LB4D2:
    LDA #$01
    STA $0706
    LDA $0704
    BEQ LB4E4
    LDY #$05
    LDA $047D
    BEQ LB4E4
    INY
LB4E4:
    LDA $B424,Y
    STA $0709
    LDA $B42B,Y
    STA $070A
    LDA $B439,Y
    STA $0433
    LDA $B432,Y
    STA $9F
    LDA $0704
    BEQ LB511
    LDA #$04
    STA $FF
    LDA $CE
    CMP #$14
    BCS LB51C
    LDA #$00
    STA $9F
    JMP LB51C
LB511:
    LDA #$01
    LDY $0754
    BEQ LB51A
    LDA #$80
LB51A:
    STA $FF
LB51C:
    LDY #$00
    STY $00
    LDA $1D
    BEQ LB52D
    LDA $0700
    CMP #$19
    BCS LB55E
    BCC LB545
LB52D:
    INY
    LDA $074E
    BEQ LB545
    DEY
    LDA $0C
    CMP $45
    BNE LB545
    LDA $0A
    AND #$40
    BNE LB559
    LDA $0783
    BNE LB55E
LB545:
    INY
    INC $00
    LDA $0703
    BNE LB554
    LDA $0700
    CMP #$21
    BCC LB55E
LB554:
    INC $00
    JMP LB55E
LB559:
    LDA #$0A
    STA $0783
LB55E:
    LDA $B440,Y
    STA $0450
    LDA $0E
    CMP #$07
    BNE LB56C
    LDY #$03
LB56C:
    LDA $B443,Y
    STA $0456
    LDY $00
    LDA $B447,Y
    STA $0702
    LDA #$00
    STA $0701
    LDA $33
    CMP $45
    BEQ LB58B
    ASL $0702
    ROL $0701
LB58B:
    RTS
    .byte $02, $04, $07
LB58F:
    LDY #$00
    LDA $0700
    CMP #$1C
    BCS LB5AD
    INY
    CMP #$0E
    BCS LB59E
    INY
LB59E:
    LDA $06FC
    AND #$7F
    BEQ LB5C5
    AND #$03
    CMP $45
    BNE LB5B3
    LDA #$00
LB5AD:
    STA $0703
    JMP LB5C5
LB5B3:
    LDA $0700
    CMP #$0B
    BCS LB5C5
    LDA $33
    STA $45
    LDA #$00
    STA $57
    STA $0705
LB5C5:
    LDA $B58C,Y
    STA $070C
    RTS
LB5CC:
    AND $0490
    CMP #$00
    BNE LB5DB
    LDA $57
    BEQ LB620
    BPL LB5FC
    BMI LB5DE
LB5DB:
    LSR A
    BCC LB5FC
LB5DE:
    LDA $0705
    CLC
    ADC $0702
    STA $0705
    LDA $57
    ADC $0701
    STA $57
    CMP $0456
    BMI LB617
    LDA $0456
    STA $57
    JMP LB620
LB5FC:
    LDA $0705
    SEC
    SBC $0702
    STA $0705
    LDA $57
    SBC $0701
    STA $57
    CMP $0450
    BPL LB617
    LDA $0450
    STA $57
LB617:
    CMP #$00
    BPL LB620
    EOR #$FF
    CLC
    ADC #$01
LB620:
    STA $0700
    RTS
LB624:
    LDA $0756
    CMP #$02
    BCC LB66E
    LDA $0A
    AND #$40
    BEQ LB664
    AND $0D
    BNE LB664
    LDA $06CE
    AND #$01
    TAX
    LDA $24,X
    BNE LB664
    LDY $B5
    DEY
    BNE LB664
    LDA $0714
    BNE LB664
    LDA $1D
    CMP #$03
    BEQ LB664
    LDA #$20
    STA $FF
    LDA #$02
    STA $24,X
    LDY $070C
    STY $0711
    DEY
    STY $0781
    INC $06CE
LB664:
    LDX #$00
    JSR LB689
    LDX #$01
    JSR LB689
LB66E:
    LDA $074E
    BNE LB686
    LDX #$02
LB675:
    STX $08
    JSR LB6F9
    JSR LF131
    JSR LF191
    JSR LEDE1
    DEX
    BPL LB675
LB686:
    RTS
    .byte $40, $C0
LB689:
    STX $08
    LDA $24,X
    ASL A
    BCS LB6F3
    LDY $24,X
    BEQ LB6F2
    DEY
    BEQ LB6BE
    LDA $86
    ADC #$04
    STA $8D,X
    LDA $6D
    ADC #$00
    STA $74,X
    LDA $CE
    STA $D5,X
    LDA #$01
    STA $BC,X
    LDY $33
    DEY
    LDA $B687,Y
    STA $5E,X
    LDA #$04
    STA $A6,X
    LDA #$07
    STA $04A0,X
    DEC $24,X
LB6BE:
    TXA
    CLC
    ADC #$07
    TAX
    LDA #$50
    STA $00
    LDA #$03
    STA $02
    LDA #$00
    JSR LBFD7
    JSR LBF0F
    LDX $08
    JSR LF13B
    JSR LF187
    JSR LE22D
    JSR LE1C8
    LDA $03D2
    AND #$CC
    BNE LB6EE
    JSR LD6D9
    JMP LECDE
LB6EE:
    LDA #$00
    STA $24,X
LB6F2:
    RTS
LB6F3:
    JSR LF13B
    JMP LED09
LB6F9:
    LDA $07A8,X
    AND #$01
    STA $07
    LDA $E4,X
    CMP #$F8
    BNE LB732
    LDA $0792
    BNE LB74A
LB70B:
    LDY #$00
    LDA $33
    LSR A
    BCC LB714
    LDY #$08
LB714:
    TYA
    ADC $86
    STA $9C,X
    LDA $6D
    ADC #$00
    STA $83,X
    LDA $CE
    CLC
    ADC #$08
    STA $E4,X
    LDA #$01
    STA $CB,X
    LDY $07
    LDA $B74D,Y
    STA $0792
LB732:
    LDY $07
    LDA $042C,X
    SEC
    SBC $B74B,Y
    STA $042C,X
    LDA $E4,X
    SBC #$00
    CMP #$20
    BCS LB748
    LDA #$F8
LB748:
    STA $E4,X
LB74A:
    RTS
    .byte $FF, $50, $40, $20
LB74F:
    LDA $0770
    BEQ LB7A3
    LDA $0E
    CMP #$08
    BCC LB7A3
    CMP #$0B
    BEQ LB7A3
    LDA $B5
    CMP #$02
    BCS LB7A3
    LDA $0787
    BNE LB7A3
    LDA $07F8
    ORA $07F9
    ORA $07FA
    BEQ LB79A
    LDY $07F8
    DEY
    BNE LB786
    LDA $07F9
    ORA $07FA
    BNE LB786
    LDA #$40
    STA $FC
LB786:
    LDA #$18
    STA $0787
    LDY #$23
    LDA #$FF
    STA $0139
    JSR L8F5F
    LDA #$A4
    JMP L8F06
LB79A:
    STA $0756
    JSR LD931
    INC $0759
LB7A3:
    RTS
LB7A4:
    LDA $0723
    BEQ LB7A3
    LDA $CE
    AND $B5
    BNE LB7A3
    STA $0723
    INC $06D6
    JMP LC998
LB7B8:
    LDA $074E
    BNE LB7F4
    STA $047D
    LDA $0747
    BNE LB7F4
    LDY #$04
LB7C7:
    LDA $0471,Y
    CLC
    ADC $0477,Y
    STA $02
    LDA $046B,Y
    BEQ LB7F1
    ADC #$00
    STA $01
    LDA $86
    SEC
    SBC $0471,Y
    LDA $6D
    SBC $046B,Y
    BMI LB7F1
    LDA $02
    SEC
    SBC $86
    LDA $01
    SBC $6D
    BPL LB7F5
LB7F1:
    DEY
    BPL LB7C7
LB7F4:
    RTS
LB7F5:
    LDA $0477,Y
    LSR A
    STA $00
    LDA $0471,Y
    CLC
    ADC $00
    STA $01
    LDA $046B,Y
    ADC #$00
    STA $00
    LDA $09
    LSR A
    BCC LB83B
    LDA $01
    SEC
    SBC $86
    LDA $00
    SBC $6D
    BPL LB828
    LDA $86
    SEC
    SBC #$01
    STA $86
    LDA $6D
    SBC #$00
    JMP LB839
LB828:
    LDA $0490
    LSR A
    BCC LB83B
    LDA $86
    CLC
    ADC #$01
    STA $86
    LDA $6D
    ADC #$00
LB839:
    STA $6D
LB83B:
    LDA #$10
    STA $00
    LDA #$01
    STA $047D
    STA $02
    LSR A
    TAX
    JMP LBFD7
    .byte $05, $02, $08, $04, $01, $03, $03, $04, $04, $04
LB855:
    LDX #$05
    STX $08
    LDA $16,X
    CMP #$30
    BNE LB8B5
    LDA $0E
    CMP #$04
    BNE LB896
    LDA $1D
    CMP #$03
    BNE LB896
    LDA $CF,X
    CMP #$AA
    BCS LB899
    LDA $CE
    CMP #$A2
    BCS LB899
    LDA $0417,X
    ADC #$FF
    STA $0417,X
    LDA $CF,X
    ADC #$01
    STA $CF,X
    LDA $010E
    SEC
    SBC #$FF
    STA $010E
    LDA $010D
    SBC #$01
    STA $010D
LB896:
    JMP LB8AC
LB899:
    LDY $010F
    LDA $B84B,Y
    LDX $B850,Y
    STA $0134,X
    JSR LBC27
    LDA #$05
    STA $0E
LB8AC:
    JSR LF1AF
    JSR LF152
    JSR LE54B
LB8B5:
    RTS
    .byte $08, $10, $08, $00
LB8BA:
    JSR LF1AF
    LDA $0747
    BNE LB902
    LDA $070E
    BEQ LB902
    TAY
    DEY
    TYA
    AND #$02
    BNE LB8D5
    INC $CE
    INC $CE
    JMP LB8D9
LB8D5:
    DEC $CE
    DEC $CE
LB8D9:
    LDA $58,X
    CLC
    ADC $B8B6,Y
    STA $CF,X
    CPY #$01
    BCC LB8F4
    LDA $0A
    AND #$80
    BEQ LB8F4
    AND $0D
    BNE LB8F4
    LDA #$F4
    STA $06DB
LB8F4:
    CPY #$03
    BNE LB902
    LDA $06DB
    STA $9F
    LDA #$00
    STA $070E
LB902:
    JSR LF152
    JSR LE87D
    JSR LD67A
    LDA $070E
    BEQ LB91D
    LDA $0786
    BNE LB91D
    LDA #$04
    STA $0786
    INC $070E
LB91D:
    RTS
LB91E:
    LDA #$2F
    STA $16,X
    LDA #$01
    STA $0F,X
    LDA $0076,Y
    STA $6E,X
    LDA $008F,Y
    STA $87,X
    LDA $00D7,Y
    STA $CF,X
    LDY $0398
    BNE LB93D
    STA $039D
LB93D:
    TXA
    STA $039A,Y
    INC $0398
    LDA #$04
    STA $FE
    RTS
    .byte $30, $60
LB94B:
    CPX #$05
    BNE LB9B7
    LDY $0398
    DEY
    LDA $0399
    CMP $B949,Y
    BEQ LB96A
    LDA $09
    LSR A
    LSR A
    BCC LB96A
    LDA $D4
    SBC #$01
    STA $D4
    INC $0399
LB96A:
    LDA $0399
    CMP #$08
    BCC LB9B7
    JSR LF152
    JSR LF1AF
    LDY #$00
LB979:
    JSR LE435
    INY
    CPY $0398
    BNE LB979
    LDA $03D1
    AND #$0C
    BEQ LB999
    DEY
LB98A:
    LDX $039A,Y
    JSR LC998
    DEY
    BPL LB98A
    STA $0398
    STA $0399
LB999:
    LDA $0399
    CMP #$20
    BCC LB9B7
    LDX #$06
    LDA #$01
    LDY #$1B
    JSR LE3F0
    LDY $02
    CPY #$D0
    BCS LB9B7
    LDA ($06),Y
    BNE LB9B7
    LDA #$26
    STA ($06),Y
LB9B7:
    LDX $08
    RTS
    .byte $0F, $07
LB9BC:
    LDA $074E
    BEQ LBA30
    LDX #$02
LB9C3:
    STX $08
    LDA $0F,X
    BNE LBA1A
    LDA $07A8,X
    LDY $06CC
    AND $B9BA,Y
    CMP #$06
    BCS LBA1A
    TAY
    LDA $046B,Y
    BEQ LBA1A
    LDA $047D,Y
    BEQ LB9E9
    SBC #$00
    STA $047D,Y
    JMP LBA1A
LB9E9:
    LDA $0747
    BNE LBA1A
    LDA #$0E
    STA $047D,Y
    LDA $046B,Y
    STA $6E,X
    LDA $0471,Y
    STA $87,X
    LDA $0477,Y
    SEC
    SBC #$08
    STA $CF,X
    LDA #$01
    STA $B6,X
    STA $0F,X
    LSR A
    STA $1E,X
    LDA #$09
    STA $049A,X
    LDA #$33
    STA $16,X
    JMP LBA2D
LBA1A:
    LDA $16,X
    CMP #$33
    BNE LBA2D
    JSR LD67A
    LDA $0F,X
    BEQ LBA2D
    JSR LF1AF
    JSR LBA33
LBA2D:
    DEX
    BPL LB9C3
LBA30:
    RTS
    .byte $18, $E8
LBA33:
    LDA $0747
    BNE LBA76
    LDA $1E,X
    BNE LBA6A
    LDA $03D1
    AND #$0C
    CMP #$0C
    BEQ LBA85
    LDY #$01
    JSR LE143
    BMI LBA4D
    INY
LBA4D:
    STY $46,X
    DEY
    LDA $BA31,Y
    STA $58,X
    LDA $00
    ADC #$28
    CMP #$50
    BCC LBA85
    LDA #$01
    STA $1E,X
    LDA #$0A
    STA $078A,X
    LDA #$08
    STA $FE
LBA6A:
    LDA $1E,X
    AND #$20
    BEQ LBA73
    JSR LBF63
LBA73:
    JSR LBF02
LBA76:
    JSR LF1AF
    JSR LF152
    JSR LE243
    JSR LD853
    JMP LE87D
LBA85:
    JSR LC998
    RTS
    .byte $04, $04, $04, $05, $05, $05, $06, $06, $06, $10, $F0
LBA94:
    LDA $07A8
    AND #$07
    BNE LBAA0
    LDA $07A8
    AND #$08
LBAA0:
    TAY
    LDA $002A,Y
    BNE LBABF
    LDX $BA89,Y
    LDA $0F,X
    BNE LBABF
    LDX $08
    TXA
    STA $06AE,Y
    LDA #$90
    STA $002A,Y
    LDA #$07
    STA $04A2,Y
    SEC
    RTS
LBABF:
    LDX $08
    CLC
    RTS
LBAC3:
    LDA $0747
    BNE LBB2B
    LDA $2A,X
    AND #$7F
    LDY $06AE,X
    CMP #$02
    BEQ LBAF3
    BCS LBB09
    TXA
    CLC
    ADC #$0D
    TAX
    LDA #$10
    STA $00
    LDA #$0F
    STA $01
    LDA #$04
    STA $02
    LDA #$00
    JSR LBFD7
    JSR LBF0F
    LDX $08
    JMP LBB28
LBAF3:
    LDA #$FE
    STA $AC,X
    LDA $001E,Y
    AND #$F7
    STA $001E,Y
    LDX $46,Y
    DEX
    LDA $BA92,X
    LDX $08
    STA $64,X
LBB09:
    DEC $2A,X
    LDA $0087,Y
    CLC
    ADC #$02
    STA $93,X
    LDA $006E,Y
    ADC #$00
    STA $7A,X
    LDA $00CF,Y
    SEC
    SBC #$0A
    STA $DB,X
    LDA #$01
    STA $C2,X
    BNE LBB2B
LBB28:
    JSR LD7C4
LBB2B:
    JSR LF19B
    JSR LF148
    JSR LE236
    JSR LE4DC
    RTS
LBB38:
    JSR LBB84
    LDA $76,X
    STA $007A,Y
    LDA $8F,X
    ORA #$05
    STA $0093,Y
    LDA $D7,X
    SBC #$10
    STA $00DB,Y
    JMP LBB6C
LBB51:
    JSR LBB84
    LDA $03EA,X
    STA $007A,Y
    LDA $06
    ASL A
    ASL A
    ASL A
    ASL A
    ORA #$05
    STA $0093,Y
    LDA $02
    ADC #$20
    STA $00DB,Y
LBB6C:
    LDA #$FB
    STA $00AC,Y
    LDA #$01
    STA $00C2,Y
    STA $002A,Y
    STA $FE
    STX $08
    JSR LBBFE
    INC $0748
    RTS
LBB84:
    LDY #$08
LBB86:
    LDA $002A,Y
    BEQ LBB92
    DEY
    CPY #$05
    BNE LBB86
    LDY #$08
LBB92:
    STY $06B7
    RTS
LBB96:
    LDX #$08
LBB98:
    STX $08
    LDA $2A,X
    BEQ LBBF4
    ASL A
    BCC LBBA7
    JSR LBAC3
    JMP LBBF4
LBBA7:
    LDY $2A,X
    DEY
    BEQ LBBC9
    INC $2A,X
    LDA $93,X
    CLC
    ADC $0775
    STA $93,X
    LDA $7A,X
    ADC #$00
    STA $7A,X
    LDA $2A,X
    CMP #$30
    BNE LBBE8
    LDA #$00
    STA $2A,X
    JMP LBBF4
LBBC9:
    TXA
    CLC
    ADC #$0D
    TAX
    LDA #$50
    STA $00
    LDA #$06
    STA $02
    LSR A
    STA $01
    LDA #$00
    JSR LBFD7
    LDX $08
    LDA $AC,X
    CMP #$05
    BNE LBBE8
    INC $2A,X
LBBE8:
    JSR LF148
    JSR LF19B
    JSR LE236
    JSR LE686
LBBF4:
    DEX
    BPL LBB98
    RTS
    .byte $17, $1D, $0B, $11, $02, $13
LBBFE:
    LDA #$01
    STA $0139
    LDX $0753
    LDY $BBF8,X
    JSR L8F5F
    INC $075E
    LDA $075E
    CMP #$64
    BNE LBC22
    LDA #$00
    STA $075E
    INC $075A
    LDA #$40
    STA $FE
LBC22:
    LDA #$02
    STA $0138
LBC27:
    LDX $0753
    LDY $BBFA,X
    JSR L8F5F
LBC30:
    LDY $0753
    LDA $BBFC,Y
LBC36:
    JSR L8F06
    LDY $0300
    LDA $02FB,Y
    BNE LBC46
    LDA #$24
    STA $02FB,Y
LBC46:
    LDX $08
    RTS
LBC49:
    LDA #$2E
    STA $1B
    LDA $76,X
    STA $73
    LDA $8F,X
    STA $8C
    LDA #$01
    STA $BB
    LDA $D7,X
    SEC
    SBC #$08
    STA $D4
LBC60:
    LDA #$01
    STA $23
    STA $14
    LDA #$03
    STA $049F
    LDA $39
    CMP #$02
    BCS LBC7B
    LDA $0756
    CMP #$02
    BCC LBC79
    LSR A
LBC79:
    STA $39
LBC7B:
    LDA #$20
    STA $03CA
    LDA #$02
    STA $FE
    RTS
LBC85:
    LDX #$05
    STX $08
    LDA $23
    BEQ LBCEA
    ASL A
    BCC LBCB3
    LDA $0747
    BNE LBCD8
    LDA $39
    BEQ LBCAA
    CMP #$03
    BEQ LBCAA
    CMP #$02
    BNE LBCD8
    JSR LCAF9
    JSR LE163
    JMP LBCD8
LBCAA:
    JSR LCA77
    JSR LDFC1
    JMP LBCD8
LBCB3:
    LDA $09
    AND #$03
    BNE LBCD2
    DEC $D4
    LDA $23
    INC $23
    CMP #$11
    BCC LBCD2
    LDA #$10
    STA $58,X
    LDA #$80
    STA $23
    ASL A
    STA $03CA
    ROL A
    STA $46,X
LBCD2:
    LDA $23
    CMP #$06
    BCC LBCEA
LBCD8:
    JSR LF152
    JSR LF1AF
    JSR LE243
    JSR LE6D2
    JSR LD853
    JSR LD67A
LBCEA:
    RTS
    .byte $04, $12
LBCED:
    PHA
    LDA #$11
    LDX $03EE
    LDY $0754
    BNE LBCFA
    LDA #$12
LBCFA:
    STA $26,X
    JSR L8A6B
    LDX $03EE
    LDA $02
    STA $03E4,X
    TAY
    LDA $06
    STA $03E6,X
    LDA ($06),Y
    JSR LBDF6
    STA $00
    LDY $0754
    BNE LBD1A
    TYA
LBD1A:
    BCC LBD41
    LDY #$11
    STY $26,X
    LDA #$C4
    LDY $00
    CPY #$58
    BEQ LBD2C
    CPY #$5D
    BNE LBD41
LBD2C:
    LDA $06BC
    BNE LBD39
    LDA #$0B
    STA $079D
    INC $06BC
LBD39:
    LDA $079D
    BNE LBD40
    LDY #$C4
LBD40:
    TYA
LBD41:
    STA $03E8,X
    JSR LBD84
    LDY $02
    LDA #$23
    STA ($06),Y
    LDA #$10
    STA $0784
    PLA
    STA $05
    LDY #$00
    LDA $0714
    BNE LBD61
    LDA $0754
    BEQ LBD62
LBD61:
    INY
LBD62:
    LDA $CE
    CLC
    ADC $BCEB,Y
    AND #$F0
    STA $D7,X
    LDY $26,X
    CPY #$11
    BEQ LBD78
    JSR LBE02
    JMP LBD7B
LBD78:
    JSR LBD9B
LBD7B:
    LDA $03EE
    EOR #$01
    STA $03EE
    RTS
LBD84:
    LDA $86
    CLC
    ADC #$08
    AND #$F0
    STA $8F,X
    LDA $6D
    ADC #$00
    STA $76,X
    STA $03EA,X
    LDA $B5
    STA $BE,X
    RTS
LBD9B:
    JSR LBE1F
    LDA #$02
    STA $FF
    LDA #$00
    STA $60,X
    STA $043C,X
    STA $9F
    LDA #$FE
    STA $A8,X
    LDA $05
    JSR LBDF6
    BCC LBDE7
    TYA
    CMP #$09
    BCC LBDBD
    SBC #$05
LBDBD:
    JSR L8E04
    .word LBDD2
    .word LBB38
    .word LBB38
    .word LBDD8
    .word LBDD2
    .word LBDDF
    .word LBDD5
    .word LBB38
    .word LBDD8
LBDD2:
    LDA #$00
    .byte $2C
LBDD5:
    LDA #$02
    .byte $2C
LBDD8:
    LDA #$03
    STA $39
    JMP LBC49
LBDDF:
    LDX #$05
    LDY $03EE
    JSR LB91E
LBDE7:
    RTS
    .byte $C1, $C0, $5F, $60, $55, $56, $57, $58, $59, $5A, $5B, $5C, $5D, $5E
LBDF6:
    LDY #$0D
LBDF8:
    CMP $BDE8,Y
    BEQ LBE01
    DEY
    BPL LBDF8
    CLC
LBE01:
    RTS
LBE02:
    JSR LBE1F
    LDA #$01
    STA $03EC,X
    STA $FD
    JSR LBE41
    LDA #$FE
    STA $9F
    LDA #$05
    STA $0139
    JSR LBC27
    LDX $03EE
    RTS
LBE1F:
    LDX $03EE
    LDY $02
    BEQ LBE40
    TYA
    SEC
    SBC #$10
    STA $02
    TAY
    LDA ($06),Y
    CMP #$C2
    BNE LBE40
    LDA #$00
    STA ($06),Y
    JSR L8A4D
    LDX $03EE
    JSR LBB51
LBE40:
    RTS
LBE41:
    LDA $8F,X
    STA $03F1,X
    LDA #$F0
    STA $60,X
    STA $62,X
    LDA #$FA
    STA $A8,X
    LDA #$FC
    STA $AA,X
    LDA #$00
    STA $043C,X
    STA $043E,X
    LDA $76,X
    STA $78,X
    LDA $8F,X
    STA $91,X
    LDA $D7,X
    CLC
    ADC #$08
    STA $D9,X
    LDA #$FA
    STA $A8,X
    RTS
LBE70:
    LDA $26,X
    BEQ LBED1
    AND #$0F
    PHA
    TAY
    TXA
    CLC
    ADC #$09
    TAX
    DEY
    BEQ LBEB3
    JSR LBFA4
    JSR LBF0F
    TXA
    CLC
    ADC #$02
    TAX
    JSR LBFA4
    JSR LBF0F
    LDX $08
    JSR LF159
    JSR LF1B6
    JSR LEC53
    PLA
    LDY $BE,X
    BEQ LBED1
    PHA
    LDA #$F0
    CMP $D9,X
    BCS LBEAA
    STA $D9,X
LBEAA:
    LDA $D7,X
    CMP #$F0
    PLA
    BCC LBED1
    BCS LBECF
LBEB3:
    JSR LBFA4
    LDX $08
    JSR LF159
    JSR LF1B6
    JSR LEBD1
    LDA $D7,X
    AND #$0F
    CMP #$05
    PLA
    BCS LBED1
    LDA #$01
    STA $03EC,X
LBECF:
    LDA #$00
LBED1:
    STA $26,X
    RTS
LBED4:
    LDX #$01
LBED6:
    STX $08
    LDA $0301
    BNE LBEFE
    LDA $03EC,X
    BEQ LBEFE
    LDA $03E6,X
    STA $06
    LDA #$05
    STA $07
    LDA $03E4,X
    STA $02
    TAY
    LDA $03E8,X
    STA ($06),Y
    JSR L8A61
    LDA #$00
    STA $03EC,X
LBEFE:
    DEX
    BPL LBED6
    RTS
LBF02:
    INX
    JSR LBF0F
    LDX $08
    RTS
LBF09:
    LDA $070E
    BNE LBF4C
    TAX
LBF0F:
    LDA $57,X
    ASL A
    ASL A
    ASL A
    ASL A
    STA $01
    LDA $57,X
    LSR A
    LSR A
    LSR A
    LSR A
    CMP #$08
    BCC LBF23
    ORA #$F0
LBF23:
    STA $00
    LDY #$00
    CMP #$00
    BPL LBF2C
    DEY
LBF2C:
    STY $02
    LDA $0400,X
    CLC
    ADC $01
    STA $0400,X
    LDA #$00
    ROL A
    PHA
    ROR A
    LDA $86,X
    ADC $00
    STA $86,X
    LDA $6D,X
    ADC $02
    STA $6D,X
    PLA
    CLC
    ADC $00
LBF4C:
    RTS
LBF4D:
    LDX #$00
    LDA $0747
    BNE LBF59
    LDA $070E
    BNE LBF4C
LBF59:
    LDA $0709
    STA $00
    LDA #$04
    JMP LBFAD
LBF63:
    LDY #$3D
    LDA $1E,X
    CMP #$05
    BNE LBF6D
LBF6B:
    LDY #$20
LBF6D:
    JMP LBF94
LBF70:
    LDY #$00
    JMP LBF77
LBF75:
    LDY #$01
LBF77:
    INX
    LDA #$03
    STA $00
    LDA #$06
    STA $01
    LDA #$02
    STA $02
    TYA
    JMP LBFD1
LBF88:
    LDY #$7F
    BNE LBF8E
LBF8C:
    LDY #$0F
LBF8E:
    LDA #$02
    BNE LBF96
LBF92:
    LDY #$1C
LBF94:
    LDA #$03
LBF96:
    STY $00
    INX
    JSR LBFAD
    LDX $08
    RTS
    .byte $06, $08, $A0, $00, $2C
LBFA4:
    LDY #$01
    LDA #$50
    STA $00
    LDA $BF9F,Y
LBFAD:
    STA $02
    LDA #$00
    JMP LBFD7
LBFB4:
    LDA #$00
    .byte $2C
LBFB7:
    LDA #$01
    PHA
    LDY $16,X
    INX
    LDA #$05
    CPY #$29
    BNE LBFC5
    LDA #$09
LBFC5:
    STA $00
    LDA #$0A
    STA $01
    LDA #$03
    STA $02
    PLA
    TAY
LBFD1:
    JSR LBFD7
    LDX $08
    RTS
LBFD7:
    PHA
    LDA $0416,X
    CLC
    ADC $0433,X
    STA $0416,X
    LDY #$00
    LDA $9F,X
    BPL LBFE9
    DEY
LBFE9:
    STY $07
    ADC $CE,X
    STA $CE,X
    LDA $B5,X
    ADC $07
    STA $B5,X
    LDA $0433,X
    CLC
    ADC $00
    STA $0433,X
    LDA $9F,X
    ADC #$00
    STA $9F,X
    CMP $02
    BMI LC018
    LDA $0433,X
    CMP #$80
    BCC LC018
    LDA $02
    STA $9F,X
    LDA #$00
    STA $0433,X
LC018:
    PLA
    BEQ LC046
    LDA $02
    EOR #$FF
    TAY
    INY
    STY $07
    LDA $0433,X
    SEC
    SBC $01
    STA $0433,X
    LDA $9F,X
    SBC #$00
    STA $9F,X
    CMP $07
    BPL LC046
    LDA $0433,X
    CMP #$80
    BCS LC046
    LDA $07
    STA $9F,X
    LDA #$FF
    STA $0433,X
LC046:
    RTS
LC047:
    LDA $0F,X
    PHA
    ASL A
    BCS LC05F
    PLA
    BEQ LC053
    JMP LC882
LC053:
    LDA $071F
    AND #$07
    CMP #$07
    BEQ LC06A
    JMP LC0CC
LC05F:
    PLA
    AND #$0F
    TAY
    LDA $000F,Y
    BNE LC06A
    STA $0F,X
LC06A:
    RTS
    .byte $03, $03, $06, $06, $06, $06, $06, $06, $07, $07, $07, $05, $09, $04, $05, $06
    .byte $08, $09, $0A, $06, $0B, $10, $40, $B0, $B0, $80, $40, $40, $80, $40, $F0, $F0
    .byte $F0
LC08C:
    LDA $6D
    SEC
    SBC #$04
    STA $6D
    LDA $0725
    SEC
    SBC #$04
    STA $0725
    LDA $071A
    SEC
    SBC #$04
    STA $071A
    LDA $071B
    SEC
    SBC #$04
    STA $071B
    LDA $072A
    SEC
    SBC #$04
    STA $072A
    LDA #$00
    STA $073B
    STA $072B
    STA $0739
    STA $073A
    LDA $9BF8,Y
    STA $072C
    RTS
LC0CC:
    LDA $0745
    BEQ LC12F
    LDA $0726
    BNE LC12F
    LDY #$0B
LC0D8:
    DEY
    BMI LC12F
    LDA $075F
    CMP $C06B,Y
    BNE LC0D8
    LDA $0725
    CMP $C076,Y
    BNE LC0D8
    LDA $CE
    CMP $C081,Y
    BNE LC115
    LDA $1D
    CMP #$00
    BNE LC115
    LDA $075F
    CMP #$06
    BNE LC122
    INC $06D9
LC102:
    INC $06DA
    LDA $06DA
    CMP #$03
    BNE LC12A
    LDA $06D9
    CMP #$03
    BEQ LC122
    BNE LC11C
LC115:
    LDA $075F
    CMP #$06
    BEQ LC102
LC11C:
    JSR LC08C
    JSR LD071
LC122:
    LDA #$00
    STA $06DA
    STA $06D9
LC12A:
    LDA #$00
    STA $0745
LC12F:
    LDA $06CD
    BEQ LC144
    STA $16,X
    LDA #$01
    STA $0F,X
    LDA #$00
    STA $1E,X
    STA $06CD
    JMP LC226
LC144:
    LDY $0739
    LDA ($E9),Y
    CMP #$FF
    BNE LC150
    JMP LC216
LC150:
    AND #$0F
    CMP #$0E
    BEQ LC164
    CPX #$05
    BCC LC164
    INY
    LDA ($E9),Y
    AND #$3F
    CMP #$2E
    BEQ LC164
    RTS
LC164:
    LDA $071D
    CLC
    ADC #$30
    AND #$F0
    STA $07
    LDA $071B
    ADC #$00
    STA $06
    LDY $0739
    INY
    LDA ($E9),Y
    ASL A
    BCC LC189
    LDA $073B
    BNE LC189
    INC $073B
    INC $073A
LC189:
    DEY
    LDA ($E9),Y
    AND #$0F
    CMP #$0F
    BNE LC1AB
    LDA $073B
    BNE LC1AB
    INY
    LDA ($E9),Y
    AND #$3F
    STA $073A
    INC $0739
    INC $0739
    INC $073B
    JMP LC0CC
LC1AB:
    LDA $073A
    STA $6E,X
    LDA ($E9),Y
    AND #$F0
    STA $87,X
    CMP $071D
    LDA $6E,X
    SBC $071B
    BCS LC1CB
    LDA ($E9),Y
    AND #$0F
    CMP #$0E
    BEQ LC231
    JMP LC250
LC1CB:
    LDA $07
    CMP $87,X
    LDA $06
    SBC $6E,X
    BCC LC216
    LDA #$01
    STA $B6,X
    LDA ($E9),Y
    ASL A
    ASL A
    ASL A
    ASL A
    STA $CF,X
    CMP #$E0
    BEQ LC231
    INY
    LDA ($E9),Y
    AND #$40
    BEQ LC1F1
    LDA $06CC
    BEQ LC25E
LC1F1:
    LDA ($E9),Y
    AND #$3F
    CMP #$37
    BCC LC1FD
    CMP #$3F
    BCC LC22E
LC1FD:
    CMP #$06
    BNE LC208
    LDY $076A
    BEQ LC208
    LDA #$02
LC208:
    STA $16,X
    LDA #$01
    STA $0F,X
    JSR LC226
    LDA $0F,X
    BNE LC25E
    RTS
LC216:
    LDA $06CB
    BNE LC224
    LDA $0398
    CMP #$01
    BNE LC22D
    LDA #$2F
LC224:
    STA $16,X
LC226:
    LDA #$00
    STA $1E,X
    JSR LC26C
LC22D:
    RTS
LC22E:
    JMP LC71B
LC231:
    INY
    INY
    LDA ($E9),Y
    LSR A
    LSR A
    LSR A
    LSR A
    LSR A
    CMP $075F
    BNE LC24D
    DEY
    LDA ($E9),Y
    STA $0750
    INY
    LDA ($E9),Y
    AND #$1F
    STA $0751
LC24D:
    JMP LC25B
LC250:
    LDY $0739
    LDA ($E9),Y
    AND #$0F
    CMP #$0E
    BNE LC25E
LC25B:
    INC $0739
LC25E:
    INC $0739
    INC $0739
    LDA #$00
    STA $073B
    LDX $08
    RTS
LC26C:
    LDA $16,X
    CMP #$15
    BCS LC27F
    TAY
    LDA $CF,X
    ADC #$08
    STA $CF,X
    LDA #$01
    STA $03D8,X
    TYA
LC27F:
    JSR L8E04
    .word LC30E
    .word LC30E
    .word LC30E
    .word LC31E
    .word LC2F0
    .word LC328
    .word LC2F1
    .word LC342
    .word LC36B
    .word LC2F0
    .word LC375
    .word LC375
    .word LC2F7
    .word LC787
    .word LC7D1
    .word LC34A
    .word LC33D
    .word LC385
    .word LC7A0
    .word LC2F0
    .word LC7A0
    .word LC7A0
    .word LC7A0
    .word LC7A0
    .word LC7B8
    .word LC2F0
    .word LC2F0
    .word LC45C
    .word LC45C
    .word LC45C
    .word LC45C
    .word LC459
    .word LC2F0
    .word LC2F0
    .word LC2F0
    .word LC2F0
    .word LC7DF
    .word LC812
    .word LC83F
    .word LC845
    .word LC80B
    .word LC803
    .word LC80B
    .word LC84B
    .word LC857
    .word LC549
    .word LBC60
    .word LB91E
    .word LC2F0
    .word LC2F0
    .word LC2F0
    .word LC2F0
    .word LC2F0
    .word LC307
    .word LC881
LC2F0:
    RTS
LC2F1:
    JSR LC30E
    JMP LC346
LC2F7:
    LDA #$02
    STA $B6,X
    STA $CF,X
    LSR A
    STA $0796,X
    LSR A
    STA $1E,X
    JMP LC346
LC307:
    LDA #$B8
    STA $CF,X
    RTS
    .byte $F8, $F4
LC30E:
    LDY #$01
    LDA $076A
    BNE LC316
    DEY
LC316:
    LDA $C30C,Y
LC319:
    STA $58,X
    JMP LC35A
LC31E:
    JSR LC30E
    LDA #$01
    STA $1E,X
    RTS
    .byte $80, $50
LC328:
    LDA #$00
    STA $03A2,X
    STA $58,X
    LDY $06CC
    LDA $C326,Y
    STA $0796,X
    LDA #$0B
    JMP LC35C
LC33D:
    LDA #$00
    JMP LC319
LC342:
    LDA #$00
    STA $58,X
LC346:
    LDA #$09
    BNE LC35C
LC34A:
    LDY #$30
    LDA $CF,X
    STA $0401,X
    BPL LC355
    LDY #$E0
LC355:
    TYA
    ADC $CF,X
    STA $58,X
LC35A:
    LDA #$03
LC35C:
    STA $049A,X
    LDA #$02
    STA $46,X
LC363:
    LDA #$00
    STA $A0,X
    STA $0434,X
    RTS
LC36B:
    LDA #$02
    STA $46,X
    LDA #$09
    STA $049A,X
    RTS
LC375:
    JSR LC346
    LDA $07A7,X
    AND #$10
    STA $58,X
    LDA $CF,X
    STA $0434,X
    RTS
LC385:
    LDA $06CB
    BNE LC395
LC38A:
    LDA #$00
    STA $06D1
    JSR LC33D
    JMP LC7D9
LC395:
    JMP LC998
    .byte $26, $2C, $32, $38, $20, $22, $24, $26, $13, $14, $15, $16
LC3A4:
    LDA $078F
    BNE LC3E5
    CPX #$05
    BCS LC3E5
    LDA #$80
    STA $078F
    LDY #$04
LC3B4:
    LDA $0016,Y
    CMP #$11
    BEQ LC3E6
    DEY
    BPL LC3B4
    INC $06D1
    LDA $06D1
    CMP #$07
    BCC LC3E5
    LDX #$04
LC3CA:
    LDA $0F,X
    BEQ LC3D3
    DEX
    BPL LC3CA
    BMI LC3E3
LC3D3:
    LDA #$00
    STA $1E,X
    LDA #$11
    STA $16,X
    JSR LC38A
    LDA #$20
    JSR LC5D8
LC3E3:
    LDX $08
LC3E5:
    RTS
LC3E6:
    LDA $CE
    CMP #$2C
    BCC LC3E5
    LDA $001E,Y
    BNE LC3E5
    LDA $006E,Y
    STA $6E,X
    LDA $0087,Y
    STA $87,X
    LDA #$01
    STA $B6,X
    LDA $00CF,Y
    SEC
    SBC #$08
    STA $CF,X
    LDA $07A7,X
    AND #$03
    TAY
    LDX #$02
LC40F:
    LDA $C398,Y
    STA $01,X
    INY
    INY
    INY
    INY
    DEX
    BPL LC40F
    LDX $08
    JSR LCF6C
    LDY $57
    CPY #$08
    BCS LC434
    TAY
    LDA $07A8,X
    AND #$03
    BEQ LC433
    TYA
    EOR #$FF
    TAY
    INY
LC433:
    TYA
LC434:
    JSR LC346
    LDY #$02
    STA $58,X
    CMP #$00
    BMI LC440
    DEY
LC440:
    STY $46,X
    LDA #$FD
    STA $A0,X
    LDA #$01
    STA $0F,X
    LDA #$05
    STA $1E,X
LC44E:
    RTS
    .byte $28, $38, $28, $38, $28, $00, $00, $10, $10, $00
LC459:
    JSR LC575
LC45C:
    LDA #$00
    STA $58,X
    LDA $16,X
    SEC
    SBC #$1B
    TAY
    LDA $C44F,Y
    STA $0388,X
    LDA $C454,Y
    STA $34,X
    LDA $CF,X
    CLC
    ADC #$04
    STA $CF,X
    LDA $87,X
    CLC
    ADC #$04
    STA $87,X
    LDA $6E,X
    ADC #$00
    STA $6E,X
    JMP LC7D9
    .byte $80, $30, $40, $80, $30, $50, $50, $70, $20, $40, $80, $A0, $70, $40, $90, $68
    .byte $0E, $05, $06, $0E, $1C, $20, $10, $0C, $1E, $22, $18, $14, $10, $60, $20, $48
LC4A8:
    LDA $078F
    BNE LC44E
    JSR LC346
    LDA $07A8,X
    AND #$03
    TAY
    LDA $C4A4,Y
    STA $078F
    LDY #$03
    LDA $06CC
    BEQ LC4C4
    INY
LC4C4:
    STY $00
    CPX $00
    BCS LC44E
    LDA $07A7,X
    AND #$03
    STA $00
    STA $01
    LDA #$FB
    STA $A0,X
    LDA #$00
    LDY $57
    BEQ LC4E4
    LDA #$04
    CPY #$19
    BCC LC4E4
    ASL A
LC4E4:
    PHA
    CLC
    ADC $00
    STA $00
    LDA $07A8,X
    AND #$03
    BEQ LC4F8
    LDA $07A9,X
    AND #$0F
    STA $00
LC4F8:
    PLA
    CLC
    ADC $01
    TAY
    LDA $C498,Y
    STA $58,X
    LDA #$01
    STA $46,X
    LDA $57
    BNE LC51C
    LDY $00
    TYA
    AND #$02
    BEQ LC51C
    LDA $58,X
    EOR #$FF
    CLC
    ADC #$01
    STA $58,X
    INC $46,X
LC51C:
    TYA
    AND #$02
    BEQ LC530
    LDA $86
    CLC
    ADC $C488,Y
    STA $87,X
    LDA $6D
    ADC #$00
    JMP LC53C
LC530:
    LDA $86
    SEC
    SBC $C488,Y
    STA $87,X
    LDA $6D
    SBC #$00
LC53C:
    STA $6E,X
    LDA #$01
    STA $0F,X
    STA $B6,X
    LDA #$F8
    STA $CF,X
    RTS
LC549:
    JSR LC575
    STX $0368
    LDA #$00
    STA $0363
    STA $0369
    LDA $87,X
    STA $0366
    LDA #$DF
    STA $0790
    STA $46,X
    LDA #$20
    STA $0364
    STA $078A,X
    LDA #$05
    STA $0483
    LSR A
    STA $0365
    RTS
LC575:
    LDY #$FF
LC577:
    INY
    LDA $000F,Y
    BNE LC577
    STY $06CF
    TXA
    ORA #$80
    STA $000F,Y
    LDA $6E,X
    STA $006E,Y
    LDA $87,X
    STA $0087,Y
    LDA #$01
    STA $0F,X
    STA $00B6,Y
    LDA $CF,X
    STA $00CF,Y
LC59C:
    RTS
    .byte $90, $80, $70, $90, $FF, $01
LC5A3:
    LDA $078F
    BNE LC59C
    STA $0434,X
    LDA $FD
    ORA #$02
    STA $FD
    LDY $0368
    LDA $0016,Y
    CMP #$2D
    BEQ LC5EC
    JSR LD1D9
    CLC
    ADC #$20
    LDY $06CC
    BEQ LC5C9
    SEC
    SBC #$10
LC5C9:
    STA $078F
    LDA $07A7,X
    AND #$03
    STA $0417,X
    TAY
    LDA $C59D,Y
LC5D8:
    STA $CF,X
    LDA $071D
    CLC
    ADC #$20
    STA $87,X
    LDA $071B
    ADC #$00
    STA $6E,X
    JMP LC61F
LC5EC:
    LDA $0087,Y
    SEC
    SBC #$0E
    STA $87,X
    LDA $006E,Y
    STA $6E,X
    LDA $00CF,Y
    CLC
    ADC #$08
    STA $CF,X
    LDA $07A7,X
    AND #$03
    STA $0417,X
    TAY
    LDA $C59D,Y
    LDY #$00
    CMP $CF,X
    BCC LC614
    INY
LC614:
    LDA $C5A1,Y
    STA $0434,X
    LDA #$00
    STA $06CB
LC61F:
    LDA #$08
    STA $049A,X
    LDA #$01
    STA $B6,X
    STA $0F,X
    LSR A
    STA $0401,X
    STA $1E,X
    RTS
    .byte $00, $30, $60, $60, $00, $20, $60, $40, $70, $40, $60, $30
LC63D:
    LDA $078F
    BNE LC689
    LDA #$20
    STA $078F
    DEC $06D7
    LDY #$06
LC64C:
    DEY
    LDA $0016,Y
    CMP #$31
    BNE LC64C
    LDA $0087,Y
    SEC
    SBC #$30
    PHA
    LDA $006E,Y
    SBC #$00
    STA $00
    LDA $06D7
    CLC
    ADC $001E,Y
    TAY
    PLA
    CLC
    ADC $C631,Y
    STA $87,X
    LDA $00
    ADC #$00
    STA $6E,X
    LDA $C637,Y
    STA $CF,X
    LDA #$01
    STA $B6,X
    STA $0F,X
    LSR A
    STA $58,X
    LDA #$08
    STA $A0,X
LC689:
    RTS
    .byte $01, $02, $04, $08, $10, $20, $40, $80, $40, $30, $90, $50, $20, $60, $A0, $70
    .byte $0A, $0B
LC69C:
    LDA $078F
    BNE LC710
    LDA $074E
    BNE LC6FD
    CPX #$03
    BCS LC710
    LDY #$00
    LDA $07A7,X
    CMP #$AA
    BCC LC6B4
    INY
LC6B4:
    LDA $075F
    CMP #$01
    BEQ LC6BC
    INY
LC6BC:
    TYA
    AND #$01
    TAY
    LDA $C69A,Y
LC6C3:
    STA $16,X
    LDA $06DD
    CMP #$FF
    BNE LC6D1
    LDA #$00
    STA $06DD
LC6D1:
    LDA $07A7,X
    AND #$07
LC6D6:
    TAY
    LDA $C68A,Y
    BIT $06DD
    BEQ LC6E6
    INY
    TYA
    AND #$07
    JMP LC6D6
LC6E6:
    ORA $06DD
    STA $06DD
    LDA $C692,Y
    JSR LC5D8
    STA $0417,X
    LDA #$20
    STA $078F
    JMP LC26C
LC6FD:
    LDY #$FF
LC6FF:
    INY
    CPY #$05
    BCS LC711
    LDA $000F,Y
    BEQ LC6FF
    LDA $0016,Y
    CMP #$08
    BNE LC6FF
LC710:
    RTS
LC711:
    LDA $FE
    ORA #$08
    STA $FE
    LDA #$08
    BNE LC6C3
LC71B:
    LDY #$00
    SEC
    SBC #$37
    PHA
    CMP #$04
    BCS LC730
    PHA
    LDY #$06
    LDA $076A
    BEQ LC72F
    LDY #$02
LC72F:
    PLA
LC730:
    STY $01
    LDY #$B0
    AND #$02
    BEQ LC73A
    LDY #$70
LC73A:
    STY $00
    LDA $071B
    STA $02
    LDA $071D
    STA $03
    LDY #$02
    PLA
    LSR A
    BCC LC74D
    INY
LC74D:
    STY $06D3
LC750:
    LDX #$FF
LC752:
    INX
    CPX #$05
    BCS LC784
    LDA $0F,X
    BNE LC752
    LDA $01
    STA $16,X
    LDA $02
    STA $6E,X
    LDA $03
    STA $87,X
    CLC
    ADC #$18
    STA $03
    LDA $02
    ADC #$00
    STA $02
    LDA $00
    STA $CF,X
    LDA #$01
    STA $B6,X
    STA $0F,X
    JSR LC26C
    DEC $06D3
    BNE LC750
LC784:
    JMP LC25E
LC787:
    LDA #$01
    STA $58,X
    LSR A
    STA $1E,X
    STA $A0,X
    LDA $CF,X
    STA $0434,X
    SEC
    SBC #$18
    STA $0417,X
    LDA #$09
    JMP LC7DB
LC7A0:
    LDA $16,X
    STA $06CB
    SEC
    SBC #$12
    JSR L8E04
    .word LC3A4
    .word LC7B7
    .word LC4A8
    .word LC5A3
    .word LC63D
    .word LC69C
LC7B7:
    RTS
LC7B8:
    LDY #$05
LC7BA:
    LDA $0016,Y
    CMP #$11
    BNE LC7C6
    LDA #$01
    STA $001E,Y
LC7C6:
    DEY
    BPL LC7BA
    LDA #$00
    STA $06CB
    STA $0F,X
    RTS
LC7D1:
    LDA #$02
    STA $46,X
    LDA #$F8
    STA $58,X
LC7D9:
    LDA #$03
LC7DB:
    STA $049A,X
    RTS
LC7DF:
    DEC $CF,X
    DEC $CF,X
    LDY $06CC
    BNE LC7ED
    LDY #$02
    JSR LC871
LC7ED:
    LDY #$FF
    LDA $03A0
    STA $1E,X
    BPL LC7F8
    TXA
    TAY
LC7F8:
    STY $03A0
    LDA #$00
    STA $46,X
    TAY
    JSR LC871
LC803:
    LDA #$FF
    STA $03A2,X
    JMP LC828
LC80B:
    LDA #$00
    STA $58,X
    JMP LC828
LC812:
    LDY #$40
    LDA $CF,X
    BPL LC81F
    EOR #$FF
    CLC
    ADC #$01
    LDY #$C0
LC81F:
    STA $0401,X
    TYA
    CLC
    ADC $CF,X
    STA $58,X
LC828:
    JSR LC363
LC82B:
    LDA #$05
    LDY $074E
    CPY #$03
    BEQ LC83B
    LDY $06CC
    BNE LC83B
    LDA #$06
LC83B:
    STA $049A,X
    RTS
LC83F:
    JSR LC84B
    JMP LC848
LC845:
    JSR LC857
LC848:
    JMP LC82B
LC84B:
    LDA #$10
    STA $0434,X
    LDA #$FF
    STA $A0,X
    JMP LC860
LC857:
    LDA #$F0
    STA $0434,X
    LDA #$00
    STA $A0,X
LC860:
    LDY #$01
    JSR LC871
    LDA #$04
    STA $049A,X
    RTS
    .byte $08, $0C, $F8, $00, $00, $FF
LC871:
    LDA $87,X
    CLC
    ADC $C86B,Y
    STA $87,X
    LDA $6E,X
    ADC $C86E,Y
    STA $6E,X
    RTS
LC881:
    RTS
LC882:
    LDX $08
    LDA #$00
    LDY $16,X
    CPY #$15
    BCC LC88F
    TYA
    SBC #$14
LC88F:
    JSR L8E04
    .word LC8E0
    .word LC935
    .word LD295
    .word LC8D6
    .word LC8D6
    .word LC8D6
    .word LC8D6
    .word LC947
    .word LC947
    .word LC947
    .word LC947
    .word LC947
    .word LC947
    .word LC947
    .word LC947
    .word LC8D6
    .word LC965
    .word LC965
    .word LC965
    .word LC965
    .word LC965
    .word LC965
    .word LC965
    .word LC94D
    .word LC94D
    .word LD065
    .word LBC85
    .word LB94B
    .word LC8D6
    .word LD2D9
    .word LB8BA
    .word LC8D6
    .word LB7A4
    .word LC8D7
LC8D6:
    RTS
LC8D7:
    JSR LF1AF
    JSR LF152
    JMP LE87D
LC8E0:
    LDA #$00
    STA $03C5,X
    JSR LF1AF
    JSR LF152
    JSR LE87D
    JSR LE243
    JSR LDFC1
    JSR LDA33
    JSR LD853
    LDY $0747
    BNE LC902
    JSR LC905
LC902:
    JMP LD67A
LC905:
    LDA $16,X
    JSR L8E04
    .word LCA77
    .word LCA77
    .word LCA77
    .word LCA77
    .word LCA77
    .word LC9D8
    .word LCA77
    .word LCB89
    .word LCC36
    .word LC934
    .word LCC4A
    .word LCC4A
    .word LC9B0
    .word LD3B0
    .word LCAF9
    .word LCAFF
    .word LCB25
    .word LCF28
    .word LCA77
    .word LC934
    .word LCEDF
LC934:
    RTS
LC935:
    JSR LD1EB
    JSR LF1AF
    JSR LF152
    JSR LE243
    JSR LD853
    JMP LD67A
LC947:
    JSR LCD3C
    JMP LD67A
LC94D:
    JSR LF1AF
    JSR LF152
    JSR LE24C
    JSR LDB7B
    JSR LF152
    JSR LED66
    JSR LD655
    JMP LD67A
LC965:
    JSR LF1AF
    JSR LF152
    JSR LE273
    JSR LDB45
    LDA $0747
    BNE LC979
    JSR LC982
LC979:
    JSR LF152
    JSR LE5C8
    JMP LD67A
LC982:
    LDA $16,X
    SEC
    SBC #$24
    JSR L8E04
    .word LD432
    .word LD5D3
    .word LD64F
    .word LD64F
    .word LD607
    .word LD631
    .word LD63D
LC998:
    LDA #$00
    STA $0F,X
    STA $16,X
    STA $1E,X
    STA $0110,X
    STA $0796,X
    STA $0125,X
    STA $03C5,X
    STA $078A,X
    RTS
LC9B0:
    LDA $0796,X
    BNE LC9CB
    JSR LC2F7
    LDA $07A8,X
    ORA #$80
    STA $0434,X
    AND #$0F
    ORA #$06
    STA $0796,X
    LDA #$F9
    STA $A0,X
LC9CB:
    JMP LBF92
    .byte $30, $1C, $00, $E8, $00, $18, $08, $F8, $0C, $F4
LC9D8:
    LDA $1E,X
    AND #$20
    BEQ LC9E1
    JMP LCAE5
LC9E1:
    LDA $3C,X
    BEQ LCA12
    DEC $3C,X
    LDA $03D1
    AND #$0C
    BNE LCA58
    LDA $03A2,X
    BNE LCA0A
    LDY $06CC
    LDA $C9CE,Y
    STA $03A2,X
    JSR LBA94
    BCC LCA0A
    LDA $1E,X
    ORA #$08
    STA $1E,X
    JMP LCA58
LCA0A:
    DEC $03A2,X
    JMP LCA58
    .byte $20, $37
LCA12:
    LDA $1E,X
    AND #$07
    CMP #$01
    BEQ LCA58
    LDA #$00
    STA $00
    LDY #$FA
    LDA $CF,X
    BMI LCA37
    LDY #$FD
    CMP #$70
    INC $00
    BCC LCA37
    DEC $00
    LDA $07A8,X
    AND #$01
    BNE LCA37
    LDY #$FA
LCA37:
    STY $A0,X
    LDA $1E,X
    ORA #$01
    STA $1E,X
    LDA $00
    AND $07A9,X
    TAY
    LDA $06CC
    BNE LCA4B
    TAY
LCA4B:
    LDA $CA10,Y
    STA $078A,X
    LDA $07A8,X
    ORA #$C0
    STA $3C,X
LCA58:
    LDY #$FC
    LDA $09
    AND #$40
    BNE LCA62
    LDY #$04
LCA62:
    STY $58,X
    LDY #$01
    JSR LE143
    BMI LCA75
    INY
    LDA $0796,X
    BNE LCA75
    LDA #$F8
    STA $58,X
LCA75:
    STY $46,X
LCA77:
    LDY #$00
    LDA $1E,X
    AND #$40
    BNE LCA98
    LDA $1E,X
    ASL A
    BCS LCAB4
    LDA $1E,X
    AND #$20
    BNE LCAE5
    LDA $1E,X
    AND #$07
    BEQ LCAB4
    CMP #$05
    BEQ LCA98
    CMP #$03
    BCS LCAC8
LCA98:
    JSR LBF63
    LDY #$00
    LDA $1E,X
    CMP #$02
    BEQ LCAAF
    AND #$40
    BEQ LCAB4
    LDA $16,X
    CMP #$2E
    BEQ LCAB4
    BNE LCAB2
LCAAF:
    JMP LBF02
LCAB2:
    LDY #$01
LCAB4:
    LDA $58,X
    PHA
    BPL LCABB
    INY
    INY
LCABB:
    CLC
    ADC $C9D0,Y
    STA $58,X
    JSR LBF02
    PLA
    STA $58,X
    RTS
LCAC8:
    LDA $0796,X
    BNE LCAEB
    STA $1E,X
    LDA $09
    AND #$01
    TAY
    INY
    STY $46,X
    DEY
    LDA $076A
    BEQ LCADF
    INY
    INY
LCADF:
    LDA $C9D4,Y
    STA $58,X
    RTS
LCAE5:
    JSR LBF63
    JMP LBF02
LCAEB:
    CMP #$0E
    BNE LCAF8
    LDA $16,X
    CMP #$06
    BNE LCAF8
    JSR LC998
LCAF8:
    RTS
LCAF9:
    JSR LBF92
    JMP LBF02
LCAFF:
    LDA $A0,X
    ORA $0434,X
    BNE LCB19
    STA $0417,X
    LDA $CF,X
    CMP $0401,X
    BCS LCB19
    LDA $09
    AND #$07
    BNE LCB18
    INC $CF,X
LCB18:
    RTS
LCB19:
    LDA $CF,X
    CMP $58,X
    BCC LCB22
    JMP LBF75
LCB22:
    JMP LBF70
LCB25:
    JSR LCB45
    JSR LCB66
    LDY #$01
    LDA $09
    AND #$03
    BNE LCB44
    LDA $09
    AND #$40
    BNE LCB3B
    LDY #$FF
LCB3B:
    STY $00
    LDA $CF,X
    CLC
    ADC $00
    STA $CF,X
LCB44:
    RTS
LCB45:
    LDA #$13
LCB47:
    STA $01
    LDA $09
    AND #$03
    BNE LCB5C
    LDY $58,X
    LDA $A0,X
    LSR A
    BCS LCB60
    CPY $01
    BEQ LCB5D
    INC $58,X
LCB5C:
    RTS
LCB5D:
    INC $A0,X
    RTS
LCB60:
    TYA
    BEQ LCB5D
    DEC $58,X
    RTS
LCB66:
    LDA $58,X
    PHA
    LDY #$01
    LDA $A0,X
    AND #$02
    BNE LCB7C
    LDA $58,X
    EOR #$FF
    CLC
    ADC #$01
    STA $58,X
    LDY #$02
LCB7C:
    STY $46,X
    JSR LBF02
    STA $00
    PLA
    STA $58,X
    RTS
    .byte $3F, $03
LCB89:
    LDA $1E,X
    AND #$20
    BNE LCBDC
    LDY $06CC
    LDA $07A8,X
    AND $CB87,Y
    BNE LCBAC
    TXA
    LSR A
    BCC LCBA2
    LDY $45
    BCS LCBAA
LCBA2:
    LDY #$02
    JSR LE143
    BPL LCBAA
    DEY
LCBAA:
    STY $46,X
LCBAC:
    JSR LCBDF
    LDA $CF,X
    SEC
    SBC $0434,X
    CMP #$20
    BCC LCBBB
    STA $CF,X
LCBBB:
    LDY $46,X
    DEY
    BNE LCBCE
    LDA $87,X
    CLC
    ADC $58,X
    STA $87,X
    LDA $6E,X
    ADC #$00
    STA $6E,X
    RTS
LCBCE:
    LDA $87,X
    SEC
    SBC $58,X
    STA $87,X
    LDA $6E,X
    SBC #$00
    STA $6E,X
    RTS
LCBDC:
    JMP LBF8C
LCBDF:
    LDA $A0,X
    AND #$02
    BNE LCC1C
    LDA $09
    AND #$07
    PHA
    LDA $A0,X
    LSR A
    BCS LCC04
    PLA
    BNE LCC03
    LDA $0434,X
    CLC
    ADC #$01
    STA $0434,X
    STA $58,X
    CMP #$02
    BNE LCC03
    INC $A0,X
LCC03:
    RTS
LCC04:
    PLA
    BNE LCC1B
    LDA $0434,X
    SEC
    SBC #$01
    STA $0434,X
    STA $58,X
    BNE LCC1B
    INC $A0,X
    LDA #$02
    STA $0796,X
LCC1B:
    RTS
LCC1C:
    LDA $0796,X
    BEQ LCC29
LCC21:
    LDA $09
    LSR A
    BCS LCC28
    INC $CF,X
LCC28:
    RTS
LCC29:
    LDA $CF,X
    ADC #$10
    CMP $CE
    BCC LCC21
    LDA #$00
    STA $A0,X
    RTS
LCC36:
    LDA $1E,X
    AND #$20
    BEQ LCC3F
    JMP LBF92
LCC3F:
    LDA #$E8
    STA $58,X
    JMP LBF02
    .byte $40, $80, $04, $04
LCC4A:
    LDA $1E,X
    AND #$20
    BEQ LCC53
    JMP LBF8C
LCC53:
    STA $03
    LDA $16,X
    SEC
    SBC #$0A
    TAY
    LDA $CC46,Y
    STA $02
    LDA $0401,X
    SEC
    SBC $02
    STA $0401,X
    LDA $87,X
    SBC #$00
    STA $87,X
    LDA $6E,X
    SBC #$00
    STA $6E,X
    LDA #$20
    STA $02
    CPX #$02
    BCC LCCC6
    LDA $58,X
    CMP #$10
    BCC LCC99
    LDA $0417,X
    CLC
    ADC $02
    STA $0417,X
    LDA $CF,X
    ADC $03
    STA $CF,X
    LDA $B6,X
    ADC #$00
    JMP LCCAC
LCC99:
    LDA $0417,X
    SEC
    SBC $02
    STA $0417,X
    LDA $CF,X
    SBC $03
    STA $CF,X
    LDA $B6,X
    SBC #$00
LCCAC:
    STA $B6,X
    LDY #$00
    LDA $CF,X
    SEC
    SBC $0434,X
    BPL LCCBF
    LDY #$10
    EOR #$FF
    CLC
    ADC #$01
LCCBF:
    CMP #$0F
    BCC LCCC6
    TYA
    STA $58,X
LCCC6:
    RTS
    .byte $00, $01, $03, $04, $05, $06, $07, $07, $08, $00, $03, $06, $09, $0B, $0D, $0E
    .byte $0F, $10, $00, $04, $09, $0D, $10, $13, $16, $17, $18, $00, $06, $0C, $12, $16
    .byte $1A, $1D, $1F, $20, $00, $07, $0F, $16, $1C, $21, $25, $27, $28, $00, $09, $12
    .byte $1B, $21, $27, $2C, $2F, $30, $00, $0B, $15, $1F, $27, $2E, $33, $37, $38, $00
    .byte $0C, $18, $24, $2D, $35, $3B, $3E, $40, $00, $0E, $1B, $28, $32, $3B, $42, $46
    .byte $48, $00, $0F, $1F, $2D, $38, $42, $4A, $4E, $50, $00, $11, $22, $31, $3E, $49
    .byte $51, $56, $58, $01, $03, $02, $00, $00, $09, $12, $1B, $24, $2D, $36, $3F, $48
    .byte $51, $5A, $63, $0C, $18
LCD3C:
    JSR LF1AF
    LDA $03D1
    AND #$08
    BNE LCDBA
    LDA $0747
    BNE LCD55
    LDA $0388,X
    JSR LD410
    AND #$1F
    STA $A0,X
LCD55:
    LDA $A0,X
    LDY $16,X
    CPY #$1F
    BCC LCD6A
    CMP #$08
    BEQ LCD65
    CMP #$18
    BNE LCD6A
LCD65:
    CLC
    ADC #$01
    STA $A0,X
LCD6A:
    STA $EF
    JSR LF152
    JSR LCE8E
    LDY $06E5,X
    LDA $03B9
    STA $0200,Y
    STA $07
    LDA $03AE
    STA $0203,Y
    STA $06
    LDA #$01
    STA $00
    JSR LCE08
    LDY #$05
    LDA $16,X
    CMP #$1F
    BCC LCD96
    LDY #$0B
LCD96:
    STY $ED
    LDA #$00
    STA $00
LCD9C:
    LDA $EF
    JSR LCE8E
    JSR LCDBB
    LDA $00
    CMP #$04
    BNE LCDB2
    LDY $06CF
    LDA $06E5,Y
    STA $06
LCDB2:
    INC $00
    LDA $00
    CMP $ED
    BCC LCD9C
LCDBA:
    RTS
LCDBB:
    LDA $03
    STA $05
    LDY $06
    LDA $01
    LSR $05
    BCS LCDCB
    EOR #$FF
    ADC #$01
LCDCB:
    CLC
    ADC $03AE
    STA $0203,Y
    STA $06
    CMP $03AE
    BCS LCDE2
    LDA $03AE
    SEC
    SBC $06
    JMP LCDE6
LCDE2:
    SEC
    SBC $03AE
LCDE6:
    CMP #$59
    BCC LCDEE
    LDA #$F8
    BNE LCE03
LCDEE:
    LDA $03B9
    CMP #$F8
    BEQ LCE03
    LDA $02
    LSR $05
    BCS LCDFF
    EOR #$FF
    ADC #$01
LCDFF:
    CLC
    ADC $03B9
LCE03:
    STA $0200,Y
    STA $07
LCE08:
    JSR LECED
    TYA
    PHA
    LDA $079F
    ORA $0747
    BNE LCE85
    STA $05
    LDY $B5
    DEY
    BNE LCE85
    LDY $CE
    LDA $0754
    BNE LCE28
    LDA $0714
    BEQ LCE31
LCE28:
    INC $05
    INC $05
    TYA
    CLC
    ADC #$18
    TAY
LCE31:
    TYA
LCE32:
    SEC
    SBC $07
    BPL LCE3C
    EOR #$FF
    CLC
    ADC #$01
LCE3C:
    CMP #$08
    BCS LCE5C
    LDA $06
    CMP #$F0
    BCS LCE5C
    LDA $0207
    CLC
    ADC #$04
    STA $04
    SEC
    SBC $06
    BPL LCE58
    EOR #$FF
    CLC
    ADC #$01
LCE58:
    CMP #$08
    BCC LCE6F
LCE5C:
    LDA $05
    CMP #$02
    BEQ LCE85
    LDY $05
    LDA $CE
    CLC
    ADC $CD3A,Y
    INC $05
    JMP LCE32
LCE6F:
    LDX #$01
    LDA $04
    CMP $06
    BCS LCE78
    INX
LCE78:
    STX $46
    LDX #$00
    LDA $00
    PHA
    JSR LD92C
    PLA
    STA $00
LCE85:
    PLA
    CLC
    ADC #$04
    STA $06
    LDX $08
    RTS
LCE8E:
    PHA
    AND #$0F
    CMP #$09
    BCC LCE9A
    EOR #$0F
    CLC
    ADC #$01
LCE9A:
    STA $01
    LDY $00
    LDA $CD2E,Y
    CLC
    ADC $01
    TAY
    LDA $CCC7,Y
    STA $01
    PLA
    PHA
    CLC
    ADC #$08
    AND #$0F
    CMP #$09
    BCC LCEBA
    EOR #$0F
    CLC
    ADC #$01
LCEBA:
    STA $02
    LDY $00
    LDA $CD2E,Y
    CLC
    ADC $02
    TAY
    LDA $CCC7,Y
    STA $02
    PLA
    LSR A
    LSR A
    LSR A
    TAY
    LDA $CD2A,Y
    STA $03
    RTS
    .byte $F8, $A0, $70, $BD, $00, $20, $20, $20, $00, $00
LCEDF:
    LDA $1E,X
    AND #$20
    BEQ LCEED
    LDA #$00
    STA $03C5,X
    JMP LBF92
LCEED:
    JSR LBF02
    LDY #$0D
    LDA #$05
    JSR LBF96
    LDA $0434,X
    LSR A
    LSR A
    LSR A
    LSR A
    TAY
    LDA $CF,X
    SEC
    SBC $CED5,Y
    BPL LCF0C
    EOR #$FF
    CLC
    ADC #$01
LCF0C:
    CMP #$08
    BCS LCF1E
    LDA $0434,X
    CLC
    ADC #$10
    STA $0434,X
    LSR A
    LSR A
    LSR A
    LSR A
    TAY
LCF1E:
    LDA $CEDA,Y
    STA $03C5,X
    RTS
    .byte $15, $30, $40
LCF28:
    LDA $1E,X
    AND #$20
    BEQ LCF31
    JMP LBF63
LCF31:
    LDA $1E,X
    BEQ LCF40
    LDA #$00
    STA $A0,X
    STA $06CB
    LDA #$10
    BNE LCF53
LCF40:
    LDA #$12
    STA $06CB
    LDY #$02
LCF47:
    LDA $CF25,Y
    STA $0001,Y
    DEY
    BPL LCF47
    JSR LCF6C
LCF53:
    STA $58,X
    LDY #$01
    LDA $A0,X
    AND #$01
    BNE LCF67
    LDA $58,X
    EOR #$FF
    CLC
    ADC #$01
    STA $58,X
    INY
LCF67:
    STY $46,X
    JMP LBF02
LCF6C:
    LDY #$00
    JSR LE143
    BPL LCF7D
    INY
    LDA $00
    EOR #$FF
    CLC
    ADC #$01
    STA $00
LCF7D:
    LDA $00
    CMP #$3C
    BCC LCF9F
    LDA #$3C
    STA $00
    LDA $16,X
    CMP #$11
    BNE LCF9F
    TYA
    CMP $A0,X
    BEQ LCF9F
    LDA $A0,X
    BEQ LCF9C
    DEC $58,X
    LDA $58,X
    BNE LCFDC
LCF9C:
    TYA
    STA $A0,X
LCF9F:
    LDA $00
    AND #$3C
    LSR A
    LSR A
    STA $00
    LDY #$00
    LDA $57
    BEQ LCFD1
    LDA $0775
    BEQ LCFD1
    INY
    LDA $57
    CMP #$19
    BCC LCFC1
    LDA $0775
    CMP #$02
    BCC LCFC1
    INY
LCFC1:
    LDA $16,X
    CMP #$12
    BNE LCFCB
    LDA $57
    BNE LCFD1
LCFCB:
    LDA $A0,X
    BNE LCFD1
    LDY #$00
LCFD1:
    LDA $0001,Y
    LDY $00
LCFD6:
    SEC
    SBC #$01
    DEY
    BPL LCFD6
LCFDC:
    RTS
    .byte $1A, $58, $98, $96, $94, $92, $90, $8E, $8C, $8A, $88, $86, $84, $82, $80
LCFEC:
    LDX $0368
    LDA $16,X
    CMP #$2D
    BNE LD005
    STX $08
    LDA $1E,X
    BEQ LD015
    AND #$40
    BEQ LD005
    LDA $CF,X
    CMP #$E0
    BCC LD00F
LD005:
    LDA #$80
    STA $FC
    INC $0772
    JMP LD071
LD00F:
    JSR LBF8C
    JMP LD17B
LD015:
    DEC $0364
    BNE LD05E
    LDA #$04
    STA $0364
    LDA $0363
    EOR #$01
    STA $0363
    LDA #$22
    STA $05
    LDY $0369
    LDA $CFDD,Y
    STA $04
    LDY $0300
    INY
    LDX #$0C
    JSR L8ACD
    LDX $08
    JSR L8A8F
    LDA #$08
    STA $FE
    LDA #$01
    STA $FD
    INC $0369
    LDA $0369
    CMP #$0F
    BNE LD05E
    JSR LC363
    LDA #$40
    STA $1E,X
    LDA #$80
    STA $FE
LD05E:
    JMP LD17B
    .byte $21, $41, $11, $31
LD065:
    LDA $1E,X
    AND #$20
    BEQ LD07F
    LDA $CF,X
    CMP #$E0
    BCC LD00F
LD071:
    LDX #$04
LD073:
    JSR LC998
    DEX
    BPL LD073
    STA $06CB
    LDX $08
    RTS
LD07F:
    LDA #$00
    STA $06CB
    LDA $0747
    BEQ LD08C
    JMP LD139
LD08C:
    LDA $0363
    BPL LD094
    JMP LD10F
LD094:
    DEC $0364
    BNE LD0A6
    LDA #$20
    STA $0364
    LDA $0363
    EOR #$01
    STA $0363
LD0A6:
    LDA $09
    AND #$0F
    BNE LD0B0
    LDA #$02
    STA $46,X
LD0B0:
    LDA $078A,X
    BEQ LD0D1
    JSR LE143
    BPL LD0D1
    LDA #$01
    STA $46,X
    LDA #$02
    STA $0365
    LDA #$20
    STA $078A,X
    STA $0790
    LDA $87,X
    CMP #$C8
    BCS LD10F
LD0D1:
    LDA $09
    AND #$03
    BNE LD10F
    LDA $87,X
    CMP $0366
    BNE LD0EA
    LDA $07A7,X
    AND #$03
    TAY
    LDA $D061,Y
    STA $06DC
LD0EA:
    LDA $87,X
    CLC
    ADC $0365
    STA $87,X
    LDY $46,X
    CPY #$01
    BEQ LD10F
    LDY #$FF
    SEC
    SBC $0366
    BPL LD107
    EOR #$FF
    CLC
    ADC #$01
    LDY #$01
LD107:
    CMP $06DC
    BCC LD10F
    STY $0365
LD10F:
    LDA $078A,X
    BNE LD13C
    JSR LBF8C
    LDA $075F
    CMP #$05
    BCC LD127
    LDA $09
    AND #$03
    BNE LD127
    JSR LBA94
LD127:
    LDA $CF,X
    CMP #$80
    BCC LD149
    LDA $07A7,X
    AND #$03
    TAY
    LDA $D061,Y
    STA $078A,X
LD139:
    JMP LD149
LD13C:
    CMP #$01
    BNE LD149
    DEC $CF,X
    JSR LC363
    LDA #$FE
    STA $A0,X
LD149:
    LDA $075F
    CMP #$07
    BEQ LD154
    CMP #$05
    BCS LD17B
LD154:
    LDA $0790
    BNE LD17B
    LDA #$20
    STA $0790
    LDA $0363
    EOR #$80
    STA $0363
    BMI LD149
    JSR LD1D9
    LDY $06CC
    BEQ LD173
    SEC
    SBC #$10
LD173:
    STA $0790
    LDA #$15
    STA $06CB
LD17B:
    JSR LD1BC
    LDY #$10
    LDA $46,X
    LSR A
    BCC LD187
    LDY #$F0
LD187:
    TYA
    CLC
    ADC $87,X
    LDY $06CF
    STA $0087,Y
    LDA $CF,X
    CLC
    ADC #$08
    STA $00CF,Y
    LDA $1E,X
    STA $001E,Y
    LDA $46,X
    STA $0046,Y
    LDA $08
    PHA
    LDX $06CF
    STX $08
    LDA #$2D
    STA $16,X
    JSR LD1BC
    PLA
    STA $08
    TAX
    LDA #$00
    STA $036A
LD1BB:
    RTS
LD1BC:
    INC $036A
    JSR LC8D7
    LDA $1E,X
    BNE LD1BB
    LDA #$0A
    STA $049A,X
    JSR LE243
    JMP LD853
    .byte $BF, $40, $BF, $BF, $BF, $40, $40, $BF
LD1D9:
    LDY $0367
    INC $0367
    LDA $0367
    AND #$07
    STA $0367
    LDA $D1D1,Y
LD1EA:
    RTS
LD1EB:
    LDA $0747
    BNE LD220
    LDA #$40
    LDY $06CC
    BEQ LD1F9
    LDA #$60
LD1F9:
    STA $00
    LDA $0401,X
    SEC
    SBC $00
    STA $0401,X
    LDA $87,X
    SBC #$01
    STA $87,X
    LDA $6E,X
    SBC #$00
    STA $6E,X
    LDY $0417,X
    LDA $CF,X
    CMP $C59D,Y
    BEQ LD220
    CLC
    ADC $0434,X
    STA $CF,X
LD220:
    JSR LF152
    LDA $1E,X
    BNE LD1EA
    LDA #$51
    STA $00
    LDY #$02
    LDA $09
    AND #$02
    BEQ LD235
    LDY #$82
LD235:
    STY $01
    LDY $06E5,X
    LDX #$00
LD23C:
    LDA $03B9
    STA $0200,Y
    LDA $00
    STA $0201,Y
    INC $00
    LDA $01
    STA $0202,Y
    LDA $03AE
    STA $0203,Y
    CLC
    ADC #$08
    STA $03AE
    INY
    INY
    INY
    INY
    INX
    CPX #$03
    BCC LD23C
    LDX $08
    JSR LF1AF
    LDY $06E5,X
    LDA $03D1
    LSR A
    PHA
    BCC LD277
    LDA #$F8
    STA $020C,Y
LD277:
    PLA
    LSR A
    PHA
    BCC LD281
    LDA #$F8
    STA $0208,Y
LD281:
    PLA
    LSR A
    PHA
    BCC LD28B
    LDA #$F8
    STA $0204,Y
LD28B:
    PLA
    LSR A
    BCC LD294
    LDA #$F8
    STA $0200,Y
LD294:
    RTS
LD295:
    DEC $A0,X
    BNE LD2A5
    LDA #$08
    STA $A0,X
    INC $58,X
    LDA $58,X
    CMP #$03
    BCS LD2BD
LD2A5:
    JSR LF152
    LDA $03B9
    STA $03BA
    LDA $03AE
    STA $03AF
    LDY $06E5,X
    LDA $58,X
    JSR LED17
    RTS
LD2BD:
    LDA #$00
    STA $0F,X
    LDA #$08
    STA $FE
    LDA #$05
    STA $0138
    JMP LD336
    .byte $00, $00, $08, $08, $00, $08, $00, $08, $54, $55, $56, $57
LD2D9:
    LDA #$00
    STA $06CB
    LDA $0746
    CMP #$05
    BCS LD311
    JSR L8E04
    .word LD311
    .word LD2F2
    .word LD312
    .word LD34E
    .word LD3A2
LD2F2:
    LDY #$05
    LDA $07FA
    CMP #$01
    BEQ LD309
    LDY #$03
    CMP #$03
    BEQ LD309
    LDY #$00
    CMP #$06
    BEQ LD309
    LDA #$FF
LD309:
    STA $06D7
    STY $1E,X
LD30E:
    INC $0746
LD311:
    RTS
LD312:
    LDA $07F8
    ORA $07F9
    ORA $07FA
    BEQ LD30E
    LDA $09
    AND #$04
    BEQ LD327
    LDA #$10
    STA $FE
LD327:
    LDY #$23
    LDA #$FF
    STA $0139
    JSR L8F5F
    LDA #$05
    STA $0139
LD336:
    LDY #$0B
    LDA $0753
    BEQ LD33F
    LDY #$11
LD33F:
    JSR L8F5F
    LDA $0753
    ASL A
    ASL A
    ASL A
    ASL A
    ORA #$04
    JMP LBC36
LD34E:
    LDA $CF,X
    CMP #$72
    BCC LD359
    DEC $CF,X
    JMP LD365
LD359:
    LDA $06D7
    BEQ LD396
    BMI LD396
    LDA #$16
    STA $06CB
LD365:
    JSR LF152
    LDY $06E5,X
    LDX #$03
LD36D:
    LDA $03B9
    CLC
    ADC $D2CD,X
    STA $0200,Y
    LDA $D2D5,X
    STA $0201,Y
    LDA #$22
    STA $0202,Y
    LDA $03AE
    CLC
    ADC $D2D1,X
    STA $0203,Y
    INY
    INY
    INY
    INY
    DEX
    BPL LD36D
    LDX $08
    RTS
LD396:
    JSR LD365
    LDA #$06
    STA $0796,X
LD39E:
    INC $0746
    RTS
LD3A2:
    JSR LD365
    LDA $0796,X
    BNE LD3AF
    LDA $07B1
    BEQ LD39E
LD3AF:
    RTS
LD3B0:
    LDA $1E,X
    BNE LD40A
    LDA $078A,X
    BNE LD40A
    LDA $A0,X
    BNE LD3E0
    LDA $58,X
    BMI LD3D5
    JSR LE143
    BPL LD3CF
    LDA $00
    EOR #$FF
    CLC
    ADC #$01
    STA $00
LD3CF:
    LDA $00
    CMP #$21
    BCC LD40A
LD3D5:
    LDA $58,X
    EOR #$FF
    CLC
    ADC #$01
    STA $58,X
    INC $A0,X
LD3E0:
    LDA $0434,X
    LDY $58,X
    BPL LD3EA
    LDA $0417,X
LD3EA:
    STA $00
    LDA $09
    LSR A
    BCC LD40A
    LDA $0747
    BNE LD40A
    LDA $CF,X
    CLC
    ADC $58,X
    STA $CF,X
    CMP $00
    BNE LD40A
    LDA #$00
    STA $A0,X
    LDA #$40
    STA $078A,X
LD40A:
    LDA #$20
    STA $03C5,X
    RTS
LD410:
    STA $07
    LDA $34,X
    BNE LD424
    LDY #$18
    LDA $58,X
    CLC
    ADC $07
    STA $58,X
    LDA $A0,X
    ADC #$00
    RTS
LD424:
    LDY #$08
    LDA $58,X
    SEC
    SBC $07
    STA $58,X
    LDA $A0,X
    SBC #$00
    RTS
LD432:
    LDA $B6,X
    CMP #$03
    BNE LD43B
    JMP LC998
LD43B:
    LDA $1E,X
    BPL LD440
    RTS
LD440:
    TAY
    LDA $03A2,X
    STA $00
    LDA $46,X
    BEQ LD44D
    JMP LD5BB
LD44D:
    LDA #$2D
    CMP $CF,X
    BCC LD462
    CPY $00
    BEQ LD45F
    CLC
    ADC #$02
    STA $CF,X
    JMP LD5B1
LD45F:
    JMP LD598
LD462:
    CMP $00CF,Y
    BCC LD474
    CPX $00
    BEQ LD45F
    CLC
    ADC #$02
    STA $00CF,Y
    JMP LD5B1
LD474:
    LDA $CF,X
    PHA
    LDA $03A2,X
    BPL LD494
    LDA $0434,X
    CLC
    ADC #$05
    STA $00
    LDA $A0,X
    ADC #$00
    BMI LD4A4
    BNE LD498
    LDA $00
    CMP #$0B
    BCC LD49E
    BCS LD498
LD494:
    CMP $08
    BEQ LD4A4
LD498:
    JSR LBFB7
    JMP LD4A7
LD49E:
    JSR LD5B1
    JMP LD4A7
LD4A4:
    JSR LBFB4
LD4A7:
    LDY $1E,X
    PLA
    SEC
    SBC $CF,X
    CLC
    ADC $00CF,Y
    STA $00CF,Y
    LDA $03A2,X
    BMI LD4BD
    TAX
    JSR LDC21
LD4BD:
    LDY $08
    LDA $00A0,Y
    ORA $0434,Y
    BEQ LD53E
    LDX $0300
    CPX #$20
    BCS LD53E
    LDA $00A0,Y
    PHA
    PHA
    JSR LD541
    LDA $01
    STA $0301,X
    LDA $00
    STA $0302,X
    LDA #$02
    STA $0303,X
    LDA $00A0,Y
    BMI LD4F7
    LDA #$A2
    STA $0304,X
    LDA #$A3
    STA $0305,X
    JMP LD4FF
LD4F7:
    LDA #$24
    STA $0304,X
    STA $0305,X
LD4FF:
    LDA $001E,Y
    TAY
    PLA
    EOR #$FF
    JSR LD541
    LDA $01
    STA $0306,X
    LDA $00
    STA $0307,X
    LDA #$02
    STA $0308,X
    PLA
    BPL LD528
    LDA #$A2
    STA $0309,X
    LDA #$A3
    STA $030A,X
    JMP LD530
LD528:
    LDA #$24
    STA $0309,X
    STA $030A,X
LD530:
    LDA #$00
    STA $030B,X
    LDA $0300
    CLC
    ADC #$0A
    STA $0300
LD53E:
    LDX $08
    RTS
LD541:
    PHA
    LDA $0087,Y
    CLC
    ADC #$08
    LDX $06CC
    BNE LD550
    CLC
    ADC #$10
LD550:
    PHA
    LDA $006E,Y
    ADC #$00
    STA $02
    PLA
    AND #$F0
    LSR A
    LSR A
    LSR A
    STA $00
    LDX $CF,Y
    PLA
    BPL LD56A
    TXA
    CLC
    ADC #$08
    TAX
LD56A:
    TXA
    LDX $0300
    ASL A
    ROL A
    PHA
    ROL A
    AND #$03
    ORA #$20
    STA $01
    LDA $02
    AND #$01
    ASL A
    ASL A
    ORA $01
    STA $01
    PLA
    AND #$E0
    CLC
    ADC $00
    STA $00
    LDA $00CF,Y
    CMP #$E8
    BCC LD597
    LDA $00
    AND #$BF
    STA $00
LD597:
    RTS
LD598:
    TYA
    TAX
    JSR LF1AF
    LDA #$06
    JSR LDA11
    LDA $03AD
    STA $0117,X
    LDA $CE
    STA $011E,X
    LDA #$01
    STA $46,X
LD5B1:
    JSR LC363
    STA $00A0,Y
    STA $0434,Y
    RTS
LD5BB:
    TYA
    PHA
    JSR LBF6B
    PLA
    TAX
    JSR LBF6B
    LDX $08
    LDA $03A2,X
    BMI LD5D0
    TAX
    JSR LDC21
LD5D0:
    LDX $08
    RTS
LD5D3:
    LDA $A0,X
    ORA $0434,X
    BNE LD5EF
    STA $0417,X
    LDA $CF,X
    CMP $0401,X
    BCS LD5EF
    LDA $09
    AND #$07
    BNE LD5EC
    INC $CF,X
LD5EC:
    JMP LD5FE
LD5EF:
    LDA $CF,X
    CMP $58,X
    BCC LD5FB
    JSR LBFB7
    JMP LD5FE
LD5FB:
    JSR LBFB4
LD5FE:
    LDA $03A2,X
    BMI LD606
    JSR LDC21
LD606:
    RTS
LD607:
    LDA #$0E
    JSR LCB47
    JSR LCB66
    LDA $03A2,X
    BMI LD630
LD614:
    LDA $86
    CLC
    ADC $00
    STA $86
    LDA $6D
    LDY $00
    BMI LD626
    ADC #$00
    JMP LD628
LD626:
    SBC #$00
LD628:
    STA $6D
    STY $03A1
    JSR LDC21
LD630:
    RTS
LD631:
    LDA $03A2,X
    BMI LD63C
    JSR LBF88
    JSR LDC21
LD63C:
    RTS
LD63D:
    JSR LBF02
    STA $00
    LDA $03A2,X
    BMI LD64E
    LDA #$10
    STA $58,X
    JSR LD614
LD64E:
    RTS
LD64F:
    JSR LD65B
    JMP LD5FE
LD655:
    JSR LD65B
    JMP LD671
LD65B:
    LDA $0747
    BNE LD679
    LDA $0417,X
    CLC
    ADC $0434,X
    STA $0417,X
    LDA $CF,X
    ADC $A0,X
    STA $CF,X
    RTS
LD671:
    LDA $03A2,X
    BEQ LD679
    JSR LDC19
LD679:
    RTS
LD67A:
    LDA $16,X
    CMP #$14
    BEQ LD6D5
    LDA $071C
    LDY $16,X
    CPY #$05
    BEQ LD68D
    CPY #$0D
    BNE LD68F
LD68D:
    ADC #$38
LD68F:
    SBC #$48
    STA $01
    LDA $071A
    SBC #$00
    STA $00
    LDA $071D
    ADC #$48
    STA $03
    LDA $071B
    ADC #$00
    STA $02
    LDA $87,X
    CMP $01
    LDA $6E,X
    SBC $00
    BMI LD6D2
    LDA $87,X
    CMP $03
    LDA $6E,X
    SBC $02
    BMI LD6D5
    LDA $1E,X
    CMP #$05
    BEQ LD6D5
    CPY #$0D
    BEQ LD6D5
    CPY #$30
    BEQ LD6D5
    CPY #$31
    BEQ LD6D5
    CPY #$32
    BEQ LD6D5
LD6D2:
    JSR LC998
LD6D5:
    RTS
    .byte $FF, $FF, $FF
LD6D9:
    LDA $24,X
    BEQ LD733
    ASL A
    BCS LD733
    LDA $09
    LSR A
    BCS LD733
    TXA
    ASL A
    ASL A
    CLC
    ADC #$1C
    TAY
    LDX #$04
LD6EE:
    STX $01
    TYA
    PHA
    LDA $1E,X
    AND #$20
    BNE LD72C
    LDA $0F,X
    BEQ LD72C
    LDA $16,X
    CMP #$24
    BCC LD706
    CMP #$2B
    BCC LD72C
LD706:
    CMP #$06
    BNE LD710
    LDA $1E,X
    CMP #$02
    BCS LD72C
LD710:
    LDA $03D8,X
    BNE LD72C
    TXA
    ASL A
    ASL A
    CLC
    ADC #$04
    TAX
    JSR LE327
    LDX $08
    BCC LD72C
    LDA #$80
    STA $24,X
    LDX $01
    JSR LD73E
LD72C:
    PLA
    TAY
    LDX $01
    DEX
    BPL LD6EE
LD733:
    LDX $08
    RTS
    .byte $06, $00, $02, $12, $11, $07, $05, $2D
LD73E:
    JSR LF152
    LDX $01
    LDA $0F,X
    BPL LD752
    AND #$0F
    TAX
    LDA $16,X
    CMP #$2D
    BEQ LD75C
    LDX $01
LD752:
    LDA $16,X
    CMP #$02
    BEQ LD7C3
    CMP #$2D
    BNE LD789
LD75C:
    DEC $0483
    BNE LD7C3
    JSR LC363
    STA $58,X
    STA $06CB
    LDA #$FE
    STA $A0,X
    LDY $075F
    LDA $D736,Y
    STA $16,X
    LDA #$20
    CPY #$03
    BCS LD77D
    ORA #$03
LD77D:
    STA $1E,X
    LDA #$80
    STA $FE
    LDX $01
    LDA #$09
    BNE LD7BC
LD789:
    CMP #$08
    BEQ LD7C3
    CMP #$0C
    BEQ LD7C3
    CMP #$15
    BCS LD7C3
LD795:
    LDA $16,X
    CMP #$0D
    BNE LD7A1
    LDA $CF,X
    ADC #$18
    STA $CF,X
LD7A1:
    JSR LE01B
    LDA $1E,X
    AND #$1F
    ORA #$20
    STA $1E,X
    LDA #$02
    LDY $16,X
    CPY #$05
    BNE LD7B6
    LDA #$06
LD7B6:
    CPY #$06
    BNE LD7BC
    LDA #$01
LD7BC:
    JSR LDA11
    LDA #$08
    STA $FF
LD7C3:
    RTS
LD7C4:
    LDA $09
    LSR A
    BCC LD7FF
    LDA $0747
    ORA $03D6
    BNE LD7FF
    TXA
    ASL A
    ASL A
    CLC
    ADC #$24
    TAY
    JSR LE325
    LDX $08
    BCC LD7FA
    LDA $06BE,X
    BNE LD7FF
    LDA #$01
    STA $06BE,X
    LDA $64,X
    EOR #$FF
    CLC
    ADC #$01
    STA $64,X
    LDA $079F
    BNE LD7FF
    JMP LD92C
LD7FA:
    LDA #$00
    STA $06BE,X
LD7FF:
    RTS
LD800:
    JSR LC998
    LDA #$06
    JSR LDA11
    LDA #$20
    STA $FE
    LDA $39
    CMP #$02
    BCC LD820
    CMP #$03
    BEQ LD83A
    LDA #$23
    STA $079F
    LDA #$40
    STA $FB
    RTS
LD820:
    LDA $0756
    BEQ LD840
    CMP #$01
    BNE LD84C
    LDX $08
    LDA #$02
    STA $0756
    JSR L85F1
    LDX $08
    LDA #$0C
    JMP LD847
LD83A:
    LDA #$0B
    STA $0110,X
    RTS
LD840:
    LDA #$01
    STA $0756
    LDA #$09
LD847:
    LDY #$00
    JSR LD948
LD84C:
    RTS
    .byte $18, $E8, $30, $D0, $08, $F8
LD853:
    LDA $09
    LSR A
    BCS LD84C
    JSR LDC41
    BCS LD880
    LDA $03D8,X
    BNE LD880
    LDA $0E
    CMP #$08
    BNE LD880
    LDA $1E,X
    AND #$20
    BNE LD880
    JSR LDC52
    JSR LE325
    LDX $08
    BCS LD881
    LDA $0491,X
    AND #$FE
    STA $0491,X
LD880:
    RTS
LD881:
    LDY $16,X
    CPY #$2E
    BNE LD88A
    JMP LD800
LD88A:
    LDA $079F
    BEQ LD895
    JMP LD795
    .byte $0A, $06, $04
LD895:
    LDA $0491,X
    AND #$01
    ORA $03D8,X
    BNE LD8F8
    LDA #$01
    ORA $0491,X
    STA $0491,X
    CPY #$12
    BEQ LD8F9
    CPY #$0D
    BEQ LD92C
    CPY #$0C
    BEQ LD92C
    CPY #$33
    BEQ LD8F9
    CPY #$15
    BCS LD92C
    LDA $074E
    BEQ LD92C
    LDA $1E,X
    ASL A
    BCS LD8F9
    LDA $1E,X
    AND #$07
    CMP #$02
    BCC LD8F9
    LDA $16,X
    CMP #$06
    BEQ LD8F8
    LDA #$08
    STA $FF
    LDA $1E,X
    ORA #$80
    STA $1E,X
    JSR LDA05
    LDA $D84F,Y
    STA $58,X
    LDA #$03
    CLC
    ADC $0484
    LDY $0796,X
    CPY #$03
    BCS LD8F5
    LDA $D892,Y
LD8F5:
    JSR LDA11
LD8F8:
    RTS
LD8F9:
    LDA $9F
    BMI LD8FF
    BNE LD969
LD8FF:
    LDA $16,X
    CMP #$07
    BCC LD90E
    LDA $CE
    CLC
    ADC #$0C
    CMP $CF,X
    BCC LD969
LD90E:
    LDA $0791
    BNE LD969
    LDA $079E
    BNE LD955
    LDA $03AD
    CMP $03AE
    BCC LD923
    JMP LD9F6
LD923:
    LDA $46,X
    CMP #$01
    BNE LD92C
    JMP LD9FF
LD92C:
    LDA $079E
    BNE LD955
LD931:
    LDX $0756
    BEQ LD958
    STA $0756
    LDA #$08
    STA $079E
    ASL A
    STA $FF
    JSR L85F1
    LDA #$0A
LD946:
    LDY #$01
LD948:
    STA $0E
    STY $1D
    LDY #$FF
    STY $0747
    INY
    STY $0775
LD955:
    LDX $08
    RTS
LD958:
    STX $57
    INX
    STX $FC
    LDA #$FC
    STA $9F
    LDA #$0B
    BNE LD946
    .byte $02, $06, $05, $06
LD969:
    LDA $16,X
    CMP #$12
    BEQ LD92C
    LDA #$04
    STA $FF
    LDA $16,X
    LDY #$00
    CMP #$14
    BEQ LD996
    CMP #$08
    BEQ LD996
    CMP #$33
    BEQ LD996
    CMP #$0C
    BEQ LD996
    INY
    CMP #$05
    BEQ LD996
    INY
    CMP #$11
    BEQ LD996
    INY
    CMP #$07
    BNE LD9B3
LD996:
    LDA $D965,Y
    JSR LDA11
    LDA $46,X
    PHA
    JSR LE02F
    PLA
    STA $46,X
    LDA #$20
    STA $1E,X
    JSR LC363
    STA $58,X
    LDA #$FD
    STA $9F
    RTS
LD9B3:
    CMP #$09
    BCC LD9D4
    AND #$01
    STA $16,X
    LDY #$00
    STY $1E,X
    LDA #$03
    JSR LDA11
    JSR LC363
    JSR LDA05
    LDA $D851,Y
    STA $58,X
    JMP LD9F1
    .byte $10, $0B
LD9D4:
    LDA #$04
    STA $1E,X
    INC $0484
    LDA $0484
    CLC
    ADC $0791
    JSR LDA11
    INC $0791
    LDY $076A
    LDA $D9D2,Y
    STA $0796,X
LD9F1:
    LDA #$FC
    STA $9F
    RTS
LD9F6:
    LDA $46,X
    CMP #$01
    BNE LD9FF
    JMP LD92C
LD9FF:
    JSR LDB1C
    JMP LD92C
LDA05:
    LDY #$01
    JSR LE143
    BPL LDA0D
    INY
LDA0D:
    STY $46,X
    DEY
    RTS
LDA11:
    STA $0110,X
    LDA #$30
    STA $012C,X
    LDA $CF,X
    STA $011E,X
    LDA $03AE
    STA $0117,X
LDA24:
    RTS
    .byte $80, $40, $20, $10, $08, $04, $02, $7F, $BF, $DF, $EF, $F7, $FB, $FD
LDA33:
    LDA $09
    LSR A
    BCC LDA24
    LDA $074E
    BEQ LDA24
    LDA $16,X
    CMP #$15
    BCS LDAB1
    CMP #$11
    BEQ LDAB1
    CMP #$0D
    BEQ LDAB1
    LDA $03D8,X
    BNE LDAB1
    JSR LDC52
    DEX
    BMI LDAB1
LDA56:
    STX $01
    TYA
    PHA
    LDA $0F,X
    BEQ LDAAA
    LDA $16,X
    CMP #$15
    BCS LDAAA
    CMP #$11
    BEQ LDAAA
    CMP #$0D
    BEQ LDAAA
    LDA $03D8,X
    BNE LDAAA
    TXA
    ASL A
    ASL A
    CLC
    ADC #$04
    TAX
    JSR LE327
    LDX $08
    LDY $01
    BCC LDAA1
    LDA $1E,X
    ORA $001E,Y
    AND #$80
    BNE LDA9B
    LDA $0491,Y
    AND $DA25,X
    BNE LDAAA
    LDA $0491,Y
    ORA $DA25,X
    STA $0491,Y
LDA9B:
    JSR LDAB4
    JMP LDAAA
LDAA1:
    LDA $0491,Y
    AND $DA2C,X
    STA $0491,Y
LDAAA:
    PLA
    TAY
    LDX $01
    DEX
    BPL LDA56
LDAB1:
    LDX $08
    RTS
LDAB4:
    LDA $001E,Y
    ORA $1E,X
    AND #$20
    BNE LDAF0
    LDA $1E,X
    CMP #$06
    BCC LDAF1
    LDA $16,X
    CMP #$05
    BEQ LDAF0
    LDA $001E,Y
    ASL A
    BCC LDAD9
    LDA #$06
    JSR LDA11
    JSR LD795
    LDY $01
LDAD9:
    TYA
    TAX
    JSR LD795
    LDX $08
    LDA $0125,X
    CLC
    ADC #$04
    LDX $01
    JSR LDA11
    LDX $08
    INC $0125,X
LDAF0:
    RTS
LDAF1:
    LDA $001E,Y
    CMP #$06
    BCC LDB15
    LDA $0016,Y
    CMP #$05
    BEQ LDAF0
    JSR LD795
    LDY $01
    LDA $0125,Y
    CLC
    ADC #$04
    LDX $08
    JSR LDA11
    LDX $01
    INC $0125,X
    RTS
LDB15:
    TYA
    TAX
    JSR LDB1C
    LDX $08
LDB1C:
    LDA $16,X
    CMP #$0D
    BEQ LDB44
    CMP #$11
    BEQ LDB44
    CMP #$05
    BEQ LDB44
    CMP #$12
    BEQ LDB36
    CMP #$0E
    BEQ LDB36
    CMP #$07
    BCS LDB44
LDB36:
    LDA $58,X
    EOR #$FF
    TAY
    INY
    STY $58,X
    LDA $46,X
    EOR #$03
    STA $46,X
LDB44:
    RTS
LDB45:
    LDA #$FF
    STA $03A2,X
    LDA $0747
    BNE LDB78
    LDA $1E,X
    BMI LDB78
    LDA $16,X
    CMP #$24
    BNE LDB5F
    LDA $1E,X
    TAX
    JSR LDB5F
LDB5F:
    JSR LDC41
    BCS LDB78
    TXA
    JSR LDC54
    LDA $CF,X
    STA $00
    TXA
    PHA
    JSR LE325
    PLA
    TAX
    BCC LDB78
    JSR LDBBC
LDB78:
    LDX $08
    RTS
LDB7B:
    LDA $0747
    BNE LDBB7
    STA $03A2,X
    JSR LDC41
    BCS LDBB7
    LDA #$02
    STA $00
LDB8C:
    LDX $08
    JSR LDC52
    AND #$02
    BNE LDBB7
    LDA $04AD,Y
    CMP #$20
    BCC LDBA1
    JSR LE325
    BCS LDBBA
LDBA1:
    LDA $04AD,Y
    CLC
    ADC #$80
    STA $04AD,Y
    LDA $04AF,Y
    CLC
    ADC #$80
    STA $04AF,Y
    DEC $00
    BNE LDB8C
LDBB7:
    LDX $08
    RTS
LDBBA:
    LDX $08
LDBBC:
    LDA $04AF,Y
    SEC
    SBC $04AD
    CMP #$04
    BCS LDBCF
    LDA $9F
    BPL LDBCF
    LDA #$01
    STA $9F
LDBCF:
    LDA $04AF
    SEC
    SBC $04AD,Y
    CMP #$06
    BCS LDBF5
    LDA $9F
    BMI LDBF5
    LDA $00
    LDY $16,X
    CPY #$2B
    BEQ LDBEB
    CPY #$2C
    BEQ LDBEB
    TXA
LDBEB:
    LDX $08
    STA $03A2,X
    LDA #$00
    STA $1D
    RTS
LDBF5:
    LDA #$01
    STA $00
    LDA $04AE
    SEC
    SBC $04AC,Y
    CMP #$08
    BCC LDC11
    INC $00
    LDA $04AE,Y
    CLC
    SBC $04AC
    CMP #$09
    BCS LDC14
LDC11:
    JSR LDF4B
LDC14:
    LDX $08
    RTS
    .byte $80, $00
LDC19:
    TAY
    LDA $CF,X
    CLC
    ADC $DC16,Y
    .byte $2C
LDC21:
    LDA $CF,X
    LDY $0E
    CPY #$0B
    BEQ LDC40
    LDY $B6,X
    CPY #$01
    BNE LDC40
    SEC
    SBC #$20
    STA $CE
    TYA
    SBC #$00
    STA $B5
    LDA #$00
    STA $9F
    STA $0433
LDC40:
    RTS
LDC41:
    LDA $03D0
    CMP #$F0
    BCS LDC51
    LDY $B5
    DEY
    BNE LDC51
    LDA $CE
    CMP #$D0
LDC51:
    RTS
LDC52:
    LDA $08
LDC54:
    ASL A
    ASL A
    CLC
    ADC #$04
    TAY
    LDA $03D1
    AND #$0F
    CMP #$0F
    RTS
    .byte $20, $10
LDC64:
    LDA $0716
    BNE LDC97
    LDA $0E
    CMP #$0B
    BEQ LDC97
    CMP #$04
    BCC LDC97
    LDA #$01
    LDY $0704
    BNE LDC84
    LDA $1D
    BEQ LDC82
    CMP #$03
    BNE LDC86
LDC82:
    LDA #$02
LDC84:
    STA $1D
LDC86:
    LDA $B5
    CMP #$01
    BNE LDC97
    LDA #$FF
    STA $0490
    LDA $CE
    CMP #$CF
    BCC LDC98
LDC97:
    RTS
LDC98:
    LDY #$02
    LDA $0714
    BNE LDCAB
    LDA $0754
    BNE LDCAB
    DEY
    LDA $0704
    BNE LDCAB
    DEY
LDCAB:
    LDA $E3AD,Y
    STA $EB
    TAY
    LDX $0754
    LDA $0714
    BEQ LDCBA
    INX
LDCBA:
    LDA $CE
    CMP $DC62,X
    BCC LDCF6
    JSR LE3E9
    BEQ LDCF6
    JSR LDFA1
    BCS LDD1A
    LDY $9F
    BPL LDCF6
    LDY $04
    CPY #$04
    BCC LDCF6
    JSR LDF8F
    BCS LDCEA
    LDY $074E
    BEQ LDCF2
    LDY $0784
    BNE LDCF2
    JSR LBCED
    JMP LDCF6
LDCEA:
    CMP #$26
    BEQ LDCF2
    LDA #$02
    STA $FF
LDCF2:
    LDA #$01
    STA $9F
LDCF6:
    LDY $EB
    LDA $CE
    CMP #$CF
    BCS LDD5E
    JSR LE3E8
    JSR LDFA1
    BCS LDD1A
    PHA
    JSR LE3E8
    STA $00
    PLA
    STA $01
    BNE LDD1D
    LDA $00
    BEQ LDD5E
    JSR LDFA1
    BCC LDD1D
LDD1A:
    JMP LDE05
LDD1D:
    JSR LDF9A
    BCS LDD5E
    LDY $9F
    BMI LDD5E
    CMP #$C5
    BNE LDD2D
    JMP LDE0E
LDD2D:
    JSR LDEBD
    BEQ LDD5E
    LDY $070E
    BNE LDD5A
    LDY $04
    CPY #$05
    BCC LDD44
    LDA $45
    STA $00
    JMP LDF4B
LDD44:
    JSR LDEC4
    LDA #$F0
    AND $CE
    STA $CE
    JSR LDEE8
    LDA #$00
    STA $9F
    STA $0433
    STA $0484
LDD5A:
    LDA #$00
    STA $1D
LDD5E:
    LDY $EB
    INY
    INY
    LDA #$02
    STA $00
LDD66:
    INY
    STY $EB
    LDA $CE
    CMP #$20
    BCC LDD85
    CMP #$E4
    BCS LDD9B
    JSR LE3EC
    BEQ LDD85
    CMP #$1C
    BEQ LDD85
    CMP #$6B
    BEQ LDD85
    JSR LDF9A
    BCC LDD9C
LDD85:
    LDY $EB
    INY
    LDA $CE
    CMP #$08
    BCC LDD9B
    CMP #$D0
    BCS LDD9B
    JSR LE3EC
    BNE LDD9C
    DEC $00
    BNE LDD66
LDD9B:
    RTS
LDD9C:
    JSR LDEBD
    BEQ LDE02
    JSR LDF9A
    BCC LDDA9
    JMP LDE2E
LDDA9:
    JSR LDFA1
    BCS LDE05
    JSR LDEDD
    BCC LDDBB
    LDA $070E
    BNE LDE02
    JMP LDDFF
LDDBB:
    LDY $1D
    CPY #$00
    BNE LDDFF
    LDY $33
    DEY
    BNE LDDFF
    CMP #$6C
    BEQ LDDCE
    CMP #$1F
    BNE LDDFF
LDDCE:
    LDA $03C4
    BNE LDDD7
    LDY #$10
    STY $FF
LDDD7:
    ORA #$20
    STA $03C4
    LDA $86
    AND #$0F
    BEQ LDDF0
    LDY #$00
    LDA $071A
    BEQ LDDEA
    INY
LDDEA:
    LDA $DE03,Y
    STA $06DE
LDDF0:
    LDA $0E
    CMP #$07
    BEQ LDE02
    CMP #$08
    BNE LDE02
    LDA #$02
    STA $0E
    RTS
LDDFF:
    JSR LDF4B
LDE02:
    RTS
    .byte $A0, $34
LDE05:
    JSR LDE1C
    INC $0748
    JMP LBBFE
LDE0E:
    LDA #$00
    STA $0772
    LDA #$02
    STA $0770
    LDA #$18
    STA $57
LDE1C:
    LDY $02
    LDA #$00
    STA ($06),Y
    JMP L8A4D
    .byte $F9, $07, $FF, $00, $18, $22, $50, $68, $90
LDE2E:
    LDY $04
    CPY #$06
    BCC LDE38
    CPY #$0A
    BCC LDE39
LDE38:
    RTS
LDE39:
    CMP #$24
    BEQ LDE41
    CMP #$25
    BNE LDE7A
LDE41:
    LDA $0E
    CMP #$05
    BEQ LDE88
    LDA #$01
    STA $33
    INC $0723
    LDA $0E
    CMP #$04
    BEQ LDE73
    LDA #$33
    JSR L9716
    LDA #$80
    STA $FC
    LSR A
    STA $0713
    LDX #$04
    LDA $CE
    STA $070F
LDE68:
    CMP $DE29,X
    BCS LDE70
    DEX
    BNE LDE68
LDE70:
    STX $010F
LDE73:
    LDA #$04
    STA $0E
    JMP LDE88
LDE7A:
    CMP #$26
    BNE LDE88
    LDA $CE
    CMP #$20
    BCS LDE88
    LDA #$01
    STA $0E
LDE88:
    LDA #$03
    STA $1D
    LDA #$00
    STA $57
    STA $0705
    LDA $86
    SEC
    SBC $071C
    CMP #$10
    BCS LDEA1
    LDA #$02
    STA $33
LDEA1:
    LDY $33
    LDA $06
    ASL A
    ASL A
    ASL A
    ASL A
    CLC
    ADC $DE24,Y
    STA $86
    LDA $06
    BNE LDEBC
    LDA $071B
    CLC
    ADC $DE26,Y
    STA $6D
LDEBC:
    RTS
LDEBD:
    CMP #$5F
    BEQ LDEC3
    CMP #$60
LDEC3:
    RTS
LDEC4:
    JSR LDEDD
    BCC LDEDC
    LDA #$70
    STA $0709
    LDA #$F9
    STA $06DB
    LDA #$03
    STA $0786
    LSR A
    STA $070E
LDEDC:
    RTS
LDEDD:
    CMP #$67
    BEQ LDEE6
    CMP #$68
    CLC
    BNE LDEE7
LDEE6:
    SEC
LDEE7:
    RTS
LDEE8:
    LDA $0B
    AND #$04
    BEQ LDF4A
    LDA $00
    CMP #$11
    BNE LDF4A
    LDA $01
    CMP #$10
    BNE LDF4A
    LDA #$30
    STA $06DE
    LDA #$03
    STA $0E
    LDA #$10
    STA $FF
    LDA #$20
    STA $03C4
    LDA $06D6
    BEQ LDF4A
    AND #$03
    ASL A
    ASL A
    TAX
    LDA $86
    CMP #$60
    BCC LDF22
    INX
    CMP #$A0
    BCC LDF22
    INX
LDF22:
    LDY $87F2,X
    DEY
    STY $075F
    LDX $9CB4,Y
    LDA $9CBC,X
    STA $0750
    LDA #$80
    STA $FC
    LDA #$00
    STA $0751
    STA $0760
    STA $075C
    STA $0752
    INC $075D
    INC $0757
LDF4A:
    RTS
LDF4B:
    LDA #$00
    LDY $57
    LDX $00
    DEX
    BNE LDF5E
    INX
    CPY #$00
    BMI LDF81
    LDA #$FF
    JMP LDF66
LDF5E:
    LDX #$02
    CPY #$01
    BPL LDF81
    LDA #$01
LDF66:
    LDY #$10
    STY $0785
    LDY #$00
    STY $57
    CMP #$00
    BPL LDF74
    DEY
LDF74:
    STY $00
    CLC
    ADC $86
    STA $86
    LDA $6D
    ADC $00
    STA $6D
LDF81:
    TXA
    EOR #$FF
    AND $0490
    STA $0490
    RTS
    .byte $10, $61, $88, $C4
LDF8F:
    JSR LDFB0
    CMP $DF8B,X
    RTS
    .byte $24, $6D, $8A, $C6
LDF9A:
    JSR LDFB0
    CMP $DF96,X
    RTS
LDFA1:
    CMP #$C2
    BEQ LDFAB
    CMP #$C3
    BEQ LDFAB
    CLC
    RTS
LDFAB:
    LDA #$01
    STA $FE
    RTS
LDFB0:
    TAY
    AND #$C0
    ASL A
    ROL A
    ROL A
    TAX
    TYA
LDFB8:
    RTS
    .byte $01, $01, $02, $02, $02, $05, $10, $F0
LDFC1:
    LDA $1E,X
    AND #$20
    BNE LDFB8
    JSR LE15B
    BCC LDFB8
    LDY $16,X
    CPY #$12
    BNE LDFD8
    LDA $CF,X
    CMP #$25
    BCC LDFB8
LDFD8:
    CPY #$0E
    BNE LDFDF
    JMP LE163
LDFDF:
    CPY #$05
    BNE LDFE6
    JMP LE185
LDFE6:
    CPY #$12
    BEQ LDFF2
    CPY #$2E
    BEQ LDFF2
    CPY #$07
    BCS LE066
LDFF2:
    JSR LE1AE
    BNE LDFFA
LDFF7:
    JMP LE0E2
LDFFA:
    JSR LE1B5
    BEQ LDFF7
    CMP #$23
    BNE LE067
    LDY $02
    LDA #$00
    STA ($06),Y
    LDA $16,X
    CMP #$15
    BCS LE01B
    CMP #$06
    BNE LE016
    JSR LE18E
LE016:
    LDA #$01
    JSR LDA11
LE01B:
    CMP #$09
    BCC LE02F
    CMP #$11
    BCS LE02F
    CMP #$0A
    BCC LE02B
    CMP #$0D
    BCC LE02F
LE02B:
    AND #$01
    STA $16,X
LE02F:
    LDA $1E,X
    AND #$F0
    ORA #$02
    STA $1E,X
    DEC $CF,X
    DEC $CF,X
    LDA $16,X
    CMP #$07
    BEQ LE048
    LDA #$FD
    LDY $074E
    BNE LE04A
LE048:
    LDA #$FF
LE04A:
    STA $A0,X
    LDY #$01
    JSR LE143
    BPL LE054
    INY
LE054:
    LDA $16,X
    CMP #$33
    BEQ LE060
    CMP #$08
    BEQ LE060
    STY $46,X
LE060:
    DEY
    LDA $DFBF,Y
    STA $58,X
LE066:
    RTS
LE067:
    LDA $04
    SEC
    SBC #$08
    CMP #$05
    BCS LE0E2
    LDA $1E,X
    AND #$40
    BNE LE0CD
    LDA $1E,X
    ASL A
    BCC LE07E
LE07B:
    JMP LE0FE
LE07E:
    LDA $1E,X
    BEQ LE07B
    CMP #$05
    BEQ LE0A5
    CMP #$03
    BCS LE0A4
    LDA $1E,X
    CMP #$02
    BNE LE0A5
    LDA #$10
    LDY $16,X
    CPY #$12
    BNE LE09A
    LDA #$00
LE09A:
    STA $0796,X
    LDA #$03
    STA $1E,X
    JSR LE14F
LE0A4:
    RTS
LE0A5:
    LDA $16,X
    CMP #$06
    BEQ LE0CD
    CMP #$12
    BNE LE0BD
    LDA #$01
    STA $46,X
    LDA #$08
    STA $58,X
    LDA $09
    AND #$07
    BEQ LE0CD
LE0BD:
    LDY #$01
    JSR LE143
    BPL LE0C5
    INY
LE0C5:
    TYA
    CMP $46,X
    BNE LE0CD
    JSR LE124
LE0CD:
    JSR LE14F
    LDA $1E,X
    AND #$80
    BNE LE0DB
    LDA #$00
    STA $1E,X
    RTS
LE0DB:
    LDA $1E,X
    AND #$BF
    STA $1E,X
    RTS
LE0E2:
    LDA $16,X
    CMP #$03
    BNE LE0EC
    LDA $1E,X
    BEQ LE124
LE0EC:
    LDA $1E,X
    TAY
    ASL A
    BCC LE0F9
    LDA $1E,X
    ORA #$40
    JMP LE0FC
LE0F9:
    LDA $DFB9,Y
LE0FC:
    STA $1E,X
LE0FE:
    LDA $CF,X
    CMP #$20
    BCC LE123
    LDY #$16
    LDA #$02
    STA $EB
LE10A:
    LDA $EB
    CMP $46,X
    BNE LE11C
    LDA #$01
    JSR LE388
    BEQ LE11C
    JSR LE1B5
    BNE LE124
LE11C:
    DEC $EB
    INY
    CPY #$18
    BCC LE10A
LE123:
    RTS
LE124:
    CPX #$05
    BEQ LE131
    LDA $1E,X
    ASL A
    BCC LE131
    LDA #$02
    STA $FF
LE131:
    LDA $16,X
    CMP #$05
    BNE LE140
    LDA #$00
    STA $00
    LDY #$FA
    JMP LCA37
LE140:
    JMP LDB36
LE143:
    LDA $87,X
    SEC
    SBC $86
    STA $00
    LDA $6E,X
    SBC $6D
    RTS
LE14F:
    JSR LC363
    LDA $CF,X
    AND #$F0
    ORA #$08
    STA $CF,X
    RTS
LE15B:
    LDA $CF,X
    CLC
    ADC #$3E
    CMP #$44
    RTS
LE163:
    JSR LE15B
    BCC LE182
    LDA $A0,X
    CLC
    ADC #$02
    CMP #$03
    BCC LE182
    JSR LE1AE
    BEQ LE182
    JSR LE1B5
    BEQ LE182
    JSR LE14F
    LDA #$FD
    STA $A0,X
LE182:
    JMP LE0FE
LE185:
    JSR LE1AE
    BEQ LE1A7
    CMP #$23
    BNE LE196
LE18E:
    JSR LD795
    LDA #$FC
    STA $A0,X
    RTS
LE196:
    LDA $078A,X
    BNE LE1A7
    LDA $1E,X
    AND #$88
    STA $1E,X
    JSR LE14F
    JMP LE0FE
LE1A7:
    LDA $1E,X
    ORA #$01
    STA $1E,X
    RTS
LE1AE:
    LDA #$00
    LDY #$15
    JMP LE388
LE1B5:
    CMP #$26
    BEQ LE1C7
    CMP #$C2
    BEQ LE1C7
    CMP #$C3
    BEQ LE1C7
    CMP #$5F
    BEQ LE1C7
    CMP #$60
LE1C7:
    RTS
LE1C8:
    LDA $D5,X
    CMP #$18
    BCC LE1EF
    JSR LE39C
    BEQ LE1EF
    JSR LE1B5
    BEQ LE1EF
    LDA $A6,X
    BMI LE1F4
    LDA $3A,X
    BNE LE1F4
    LDA #$FD
    STA $A6,X
    LDA #$01
    STA $3A,X
    LDA $D5,X
    AND #$F8
    STA $D5,X
    RTS
LE1EF:
    LDA #$00
    STA $3A,X
    RTS
LE1F4:
    LDA #$80
    STA $24,X
    LDA #$02
    STA $FF
    RTS
    .byte $02, $08, $0E, $20, $03, $14, $0D, $20, $02, $14, $0E, $20, $02, $09, $0E, $15
    .byte $00, $00, $18, $06, $00, $00, $20, $0D, $00, $00, $30, $0D, $00, $00, $08, $08
    .byte $06, $04, $0A, $08, $03, $0E, $0D, $14, $00, $02, $10, $15, $04, $04, $0C, $1C
LE22D:
    TXA
    CLC
    ADC #$07
    TAX
    LDY #$02
    BNE LE23D
LE236:
    TXA
    CLC
    ADC #$09
    TAX
    LDY #$06
LE23D:
    JSR LE29C
    JMP LE2DE
LE243:
    LDY #$48
    STY $00
    LDY #$44
    JMP LE252
LE24C:
    LDY #$08
    STY $00
    LDY #$04
LE252:
    LDA $87,X
    SEC
    SBC $071C
    STA $01
    LDA $6E,X
    SBC $071A
    BMI LE267
    ORA $01
    BEQ LE267
    LDY $00
LE267:
    TYA
    AND $03D1
    STA $03D8,X
    BNE LE289
    JMP LE27C
LE273:
    INX
    JSR LF1F6
    DEX
    CMP #$FE
    BCS LE289
LE27C:
    TXA
    CLC
    ADC #$01
    TAX
    LDY #$01
    JSR LE29C
    JMP LE2DE
LE289:
    TXA
    ASL A
    ASL A
    TAY
    LDA #$FF
    STA $04B0,Y
    STA $04B1,Y
    STA $04B2,Y
    STA $04B3,Y
    RTS
LE29C:
    STX $00
    LDA $03B8,Y
    STA $02
    LDA $03AD,Y
    STA $01
    TXA
    ASL A
    ASL A
    PHA
    TAY
    LDA $0499,X
    ASL A
    ASL A
    TAX
    LDA $01
    CLC
    ADC $E1FD,X
    STA $04AC,Y
    LDA $01
    CLC
    ADC $E1FF,X
    STA $04AE,Y
    INX
    INY
    LDA $02
    CLC
    ADC $E1FD,X
    STA $04AC,Y
    LDA $02
    CLC
    ADC $E1FF,X
    STA $04AE,Y
    PLA
    TAY
    LDX $00
    RTS
LE2DE:
    LDA $071C
    CLC
    ADC #$80
    STA $02
    LDA $071A
    ADC #$00
    STA $01
    LDA $86,X
    CMP $02
    LDA $6D,X
    SBC $01
    BCC LE30C
    LDA $04AE,Y
    BMI LE309
    LDA #$FF
    LDX $04AC,Y
    BMI LE306
    STA $04AC,Y
LE306:
    STA $04AE,Y
LE309:
    LDX $08
    RTS
LE30C:
    LDA $04AC,Y
    BPL LE322
    CMP #$A0
    BCC LE322
    LDA #$00
    LDX $04AE,Y
    BPL LE31F
    STA $04AE,Y
LE31F:
    STA $04AC,Y
LE322:
    LDX $08
    RTS
LE325:
    LDX #$00
LE327:
    STY $06
    LDA #$01
    STA $07
LE32D:
    LDA $04AC,Y
    CMP $04AC,X
    BCS LE35F
    CMP $04AE,X
    BCC LE34C
    BEQ LE37E
    LDA $04AE,Y
    CMP $04AC,Y
    BCC LE37E
    CMP $04AC,X
    BCS LE37E
    LDY $06
    RTS
LE34C:
    LDA $04AE,X
    CMP $04AC,X
    BCC LE37E
    LDA $04AE,Y
    CMP $04AC,X
    BCS LE37E
    LDY $06
    RTS
LE35F:
    CMP $04AC,X
    BEQ LE37E
    CMP $04AE,X
    BCC LE37E
    BEQ LE37E
    CMP $04AE,Y
    BCC LE37A
    BEQ LE37A
    LDA $04AE,Y
    CMP $04AC,X
    BCS LE37E
LE37A:
    CLC
    LDY $06
    RTS
LE37E:
    INX
    INY
    DEC $07
    BPL LE32D
    SEC
    LDY $06
    RTS
LE388:
    PHA
    TXA
    CLC
    ADC #$01
    TAX
    PLA
    JMP LE3A5
    .byte $8A, $18, $69, $0D, $AA, $A0, $1B, $4C, $A3, $E3
LE39C:
    LDY #$1A
    TXA
    CLC
    ADC #$07
    TAX
    LDA #$00
LE3A5:
    JSR LE3F0
    LDX $08
    CMP #$00
    RTS
    .byte $00, $07, $0E, $08, $03, $0C, $02, $02, $0D, $0D, $08, $03, $0C, $02, $02, $0D
    .byte $0D, $08, $03, $0C, $02, $02, $0D, $0D, $08, $00, $10, $04, $14, $04, $04, $04
    .byte $20, $20, $08, $18, $08, $18, $02, $20, $20, $08, $18, $08, $18, $12, $20, $20
    .byte $18, $18, $18, $18, $18, $14, $14, $06, $06, $08, $10
LE3E8:
    INY
LE3E9:
    LDA #$00
    .byte $2C
LE3EC:
    LDA #$01
    LDX #$00
LE3F0:
    PHA
    STY $04
    LDA $E3B0,Y
    CLC
    ADC $86,X
    STA $05
    LDA $6D,X
    ADC #$00
    AND #$01
    LSR A
    ORA $05
    ROR A
    LSR A
    LSR A
    LSR A
    JSR L9BE1
    LDY $04
    LDA $CE,X
    CLC
    ADC $E3CC,Y
    AND #$F0
    SEC
    SBC #$20
    STA $02
    TAY
    LDA ($06),Y
    STA $03
    LDY $04
    PLA
    BNE LE429
    LDA $CE,X
    JMP LE42B
LE429:
    LDA $86,X
LE42B:
    AND #$0F
    STA $04
    LDA $03
    RTS
    .byte $FF, $00, $30
LE435:
    STY $00
    LDA $03B9
    CLC
    ADC $E433,Y
    LDX $039A,Y
    LDY $06E5,X
    STY $02
    JSR LE4AE
    LDA $03AE
    STA $0203,Y
    STA $020B,Y
    STA $0213,Y
    CLC
    ADC #$06
    STA $0207,Y
    STA $020F,Y
    STA $0217,Y
    LDA #$21
    STA $0202,Y
    STA $020A,Y
    STA $0212,Y
    ORA #$40
    STA $0206,Y
    STA $020E,Y
    STA $0216,Y
    LDX #$05
LE479:
    LDA #$E1
    STA $0201,Y
    INY
    INY
    INY
    INY
    DEX
    BPL LE479
    LDY $02
    LDA $00
    BNE LE490
    LDA #$E0
    STA $0201,Y
LE490:
    LDX #$00
LE492:
    LDA $039D
    SEC
    SBC $0200,Y
    CMP #$64
    BCC LE4A2
    LDA #$F8
    STA $0200,Y
LE4A2:
    INY
    INY
    INY
    INY
    INX
    CPX #$06
    BNE LE492
    LDY $00
    RTS
LE4AE:
    LDX #$06
LE4B0:
    STA $0200,Y
    CLC
    ADC #$08
    INY
    INY
    INY
    INY
    DEX
    BNE LE4B0
    LDY $02
    RTS
    .byte $04, $00, $04, $00, $00, $04, $00, $04, $00, $08, $00, $08, $08, $00, $08, $00
    .byte $80, $82, $81, $83, $81, $83, $80, $82, $03, $03, $C3, $C3
LE4DC:
    LDY $06F3,X
    LDA $0747
    BNE LE4EC
    LDA $2A,X
    AND #$7F
    CMP #$01
    BEQ LE4F0
LE4EC:
    LDX #$00
    BEQ LE4F7
LE4F0:
    LDA $09
    LSR A
    LSR A
    AND #$03
    TAX
LE4F7:
    LDA $03BE
    CLC
    ADC $E4C4,X
    STA $0200,Y
    CLC
    ADC $E4CC,X
    STA $0204,Y
    LDA $03B3
    CLC
    ADC $E4C0,X
    STA $0203,Y
    CLC
    ADC $E4C8,X
    STA $0207,Y
    LDA $E4D0,X
    STA $0201,Y
    LDA $E4D4,X
    STA $0205,Y
    LDA $E4D8,X
    STA $0202,Y
    STA $0206,Y
    LDX $08
    LDA $03D6
    AND #$FC
    BEQ LE540
    LDA #$00
    STA $2A,X
    LDA #$F8
    JSR LE5C1
LE540:
    RTS
    .byte $F9, $50, $F7, $50, $FA, $FB, $F8, $FB, $F6, $FB
LE54B:
    LDY $06E5,X
    LDA $03AE
    STA $0203,Y
    CLC
    ADC #$08
    STA $0207,Y
    STA $020B,Y
    CLC
    ADC #$0C
    STA $05
    LDA $CF,X
    JSR LE5C1
    ADC #$08
    STA $0208,Y
    LDA $010D
    STA $02
    LDA #$01
    STA $03
    STA $04
    STA $0202,Y
    STA $0206,Y
    STA $020A,Y
    LDA #$7E
    STA $0201,Y
    STA $0209,Y
    LDA #$7F
    STA $0205,Y
    LDA $070F
    BEQ LE5A7
    TYA
    CLC
    ADC #$0C
    TAY
    LDA $010F
    ASL A
    TAX
    LDA $E541,X
    STA $00
    LDA $E542,X
    JSR LEBB2
LE5A7:
    LDX $08
    LDY $06E5,X
    LDA $03D1
    AND #$0E
    BEQ LE5C7
LE5B3:
    LDA #$F8
LE5B5:
    STA $0214,Y
    STA $0210,Y
LE5BB:
    STA $020C,Y
LE5BE:
    STA $0208,Y
LE5C1:
    STA $0204,Y
    STA $0200,Y
LE5C7:
    RTS
LE5C8:
    LDY $06E5,X
    STY $02
    INY
    INY
    INY
    LDA $03AE
    JSR LE4AE
    LDX $08
    LDA $CF,X
    JSR LE5BB
    LDY $074E
    CPY #$03
    BEQ LE5E9
    LDY $06CC
    BEQ LE5EB
LE5E9:
    LDA #$F8
LE5EB:
    LDY $06E5,X
    STA $0210,Y
    STA $0214,Y
    LDA #$5B
    LDX $0743
    BEQ LE5FD
    LDA #$75
LE5FD:
    LDX $08
    INY
    JSR LE5B5
    LDA #$02
    INY
    JSR LE5B5
    INX
    JSR LF1F6
    DEX
    LDY $06E5,X
    ASL A
    PHA
    BCC LE61A
    LDA #$F8
    STA $0200,Y
LE61A:
    PLA
    ASL A
    PHA
    BCC LE624
    LDA #$F8
    STA $0204,Y
LE624:
    PLA
    ASL A
    PHA
    BCC LE62E
    LDA #$F8
    STA $0208,Y
LE62E:
    PLA
    ASL A
    PHA
    BCC LE638
    LDA #$F8
    STA $020C,Y
LE638:
    PLA
    ASL A
    PHA
    BCC LE642
    LDA #$F8
    STA $0210,Y
LE642:
    PLA
    ASL A
    BCC LE64B
    LDA #$F8
    STA $0214,Y
LE64B:
    LDA $03D1
    ASL A
    BCC LE654
    JSR LE5B3
LE654:
    RTS
LE655:
    LDA $09
    LSR A
    BCS LE65C
    DEC $DB,X
LE65C:
    LDA $DB,X
    JSR LE5C1
    LDA $03B3
    STA $0203,Y
    CLC
    ADC #$08
    STA $0207,Y
    LDA #$02
    STA $0202,Y
    STA $0206,Y
    LDA #$F7
    STA $0201,Y
    LDA #$FB
    STA $0205,Y
    JMP LE6BD
    .byte $60, $61, $62, $63
LE686:
    LDY $06F3,X
    LDA $2A,X
    CMP #$02
    BCS LE655
    LDA $DB,X
    STA $0200,Y
    CLC
    ADC #$08
    STA $0204,Y
    LDA $03B3
    STA $0203,Y
    STA $0207,Y
    LDA $09
    LSR A
    AND #$03
    TAX
    LDA $E682,X
    INY
    JSR LE5C1
    DEY
    LDA #$02
    STA $0202,Y
    LDA #$82
    STA $0206,Y
    LDX $08
LE6BD:
    RTS
    .byte $76, $77, $78, $79, $D6, $D6, $D9, $D9, $8D, $8D, $E4, $E4, $76, $77, $78, $79
    .byte $02, $01, $02, $01
LE6D2:
    LDY $06EA
    LDA $03B9
    CLC
    ADC #$08
    STA $02
    LDA $03AE
    STA $05
    LDX $39
    LDA $E6CE,X
    ORA $03CA
    STA $04
    TXA
    PHA
    ASL A
    ASL A
    TAX
    LDA #$01
    STA $07
    STA $03
LE6F7:
    LDA $E6BE,X
    STA $00
    LDA $E6BF,X
    JSR LEBB2
    DEC $07
    BPL LE6F7
    LDY $06EA
    PLA
    BEQ LE73B
    CMP #$03
    BEQ LE73B
    STA $00
    LDA $09
    LSR A
    AND #$03
    ORA $03CA
    STA $0202,Y
    STA $0206,Y
    LDX $00
    DEX
    BEQ LE72B
    STA $020A,Y
    STA $020E,Y
LE72B:
    LDA $0206,Y
    ORA #$40
    STA $0206,Y
    LDA $020E,Y
    ORA #$40
    STA $020E,Y
LE73B:
    JMP LEB64
    .byte $FC, $FC, $AA, $AB, $AC, $AD, $FC, $FC, $AE, $AF, $B0, $B1, $FC, $A5, $A6, $A7
    .byte $A8, $A9, $FC, $A0, $A1, $A2, $A3, $A4, $69, $A5, $6A, $A7, $A8, $A9, $6B, $A0
    .byte $6C, $A2, $A3, $A4, $FC, $FC, $96, $97, $98, $99, $FC, $FC, $9A, $9B, $9C, $9D
    .byte $FC, $FC, $8F, $8E, $8E, $8F, $FC, $FC, $95, $94, $94, $95, $FC, $FC, $DC, $DC
    .byte $DF, $DF, $DC, $DC, $DD, $DD, $DE, $DE, $FC, $FC, $B2, $B3, $B4, $B5, $FC, $FC
    .byte $B6, $B3, $B7, $B5, $FC, $FC, $70, $71, $72, $73, $FC, $FC, $6E, $6E, $6F, $6F
    .byte $FC, $FC, $6D, $6D, $6F, $6F, $FC, $FC, $6F, $6F, $6E, $6E, $FC, $FC, $6F, $6F
    .byte $6D, $6D, $FC, $FC, $F4, $F4, $F5, $F5, $FC, $FC, $F4, $F4, $F5, $F5, $FC, $FC
    .byte $F5, $F5, $F4, $F4, $FC, $FC, $F5, $F5, $F4, $F4, $FC, $FC, $FC, $FC, $EF, $EF
    .byte $B9, $B8, $BB, $BA, $BC, $BC, $FC, $FC, $BD, $BD, $BC, $BC, $7A, $7B, $DA, $DB
    .byte $D8, $D8, $CD, $CD, $CE, $CE, $CF, $CF, $7D, $7C, $D1, $8C, $D3, $D2, $7D, $7C
    .byte $89, $88, $8B, $8A, $D5, $D4, $E3, $E2, $D3, $D2, $D5, $D4, $E3, $E2, $8B, $8A
    .byte $E5, $E5, $E6, $E6, $EB, $EB, $EC, $EC, $ED, $ED, $EE, $EE, $FC, $FC, $D0, $D0
    .byte $D7, $D7, $BF, $BE, $C1, $C0, $C2, $FC, $C4, $C3, $C6, $C5, $C8, $C7, $BF, $BE
    .byte $CA, $C9, $C2, $FC, $C4, $C3, $C6, $C5, $CC, $CB, $FC, $FC, $E8, $E7, $EA, $E9
    .byte $F2, $F2, $F3, $F3, $F2, $F2, $F1, $F1, $F1, $F1, $FC, $FC, $F0, $F0, $FC, $FC
    .byte $FC, $FC, $0C, $0C, $00, $0C, $0C, $A8, $54, $3C, $EA, $18, $48, $48, $CC, $C0
    .byte $18, $18, $18, $90, $24, $FF, $48, $9C, $D2, $D8, $F0, $F6, $FC, $01, $02, $03
    .byte $02, $01, $01, $03, $03, $03, $01, $01, $02, $02, $21, $01, $02, $01, $01, $02
    .byte $FF, $02, $02, $01, $01, $02, $02, $02, $08, $18, $18, $19, $1A, $19, $18
LE87D:
    LDA $CF,X
    STA $02
    LDA $03AE
    STA $05
    LDY $06E5,X
    STY $EB
    LDA #$00
    STA $0109
    LDA $46,X
    STA $03
    LDA $03C5,X
    STA $04
    LDA $16,X
    CMP #$0D
    BNE LE8A9
    LDY $58,X
    BMI LE8A9
    LDY $078A,X
    BEQ LE8A9
    RTS
LE8A9:
    LDA $1E,X
    STA $ED
    AND #$1F
    TAY
    LDA $16,X
    CMP #$35
    BNE LE8BE
    LDY #$00
    LDA #$01
    STA $03
    LDA #$15
LE8BE:
    CMP #$33
    BNE LE8D5
    DEC $02
    LDA #$03
    LDY $078A,X
    BEQ LE8CD
    ORA #$20
LE8CD:
    STA $04
    LDY #$00
    STY $ED
    LDA #$08
LE8D5:
    CMP #$32
    BNE LE8E1
    LDY #$03
    LDX $070E
    LDA $E878,X
LE8E1:
    STA $EF
    STY $EC
    LDX $08
    CMP #$0C
    BNE LE8F2
    LDA $A0,X
    BMI LE8F2
    INC $0109
LE8F2:
    LDA $036A
    BEQ LE900
    LDY #$16
    CMP #$01
    BEQ LE8FE
    INY
LE8FE:
    STY $EF
LE900:
    LDY $EF
    CPY #$06
    BNE LE923
    LDA $1E,X
    CMP #$02
    BCC LE910
    LDX #$04
    STX $EC
LE910:
    AND #$20
    ORA $0747
    BNE LE923
    LDA $09
    AND #$08
    BNE LE923
    LDA $03
    EOR #$03
    STA $03
LE923:
    LDA $E85B,Y
    ORA $04
    STA $04
    LDA $E840,Y
    TAX
    LDY $EC
    LDA $036A
    BEQ LE965
    CMP #$01
    BNE LE94C
    LDA $0363
    BPL LE940
    LDX #$DE
LE940:
    LDA $ED
    AND #$20
    BEQ LE949
LE946:
    STX $0109
LE949:
    JMP LEA4B
LE94C:
    LDA $0363
    AND #$01
    BEQ LE955
    LDX #$E4
LE955:
    LDA $ED
    AND #$20
    BEQ LE949
    LDA $02
    SEC
    SBC #$10
    STA $02
    JMP LE946
LE965:
    CPX #$24
    BNE LE97A
    CPY #$05
    BNE LE977
    LDX #$30
    LDA #$02
    STA $03
    LDA #$05
    STA $EC
LE977:
    JMP LE9CA
LE97A:
    CPX #$90
    BNE LE990
    LDA $ED
    AND #$20
    BNE LE98D
    LDA $078F
    CMP #$10
    BCS LE98D
    LDX #$96
LE98D:
    JMP LEA37
LE990:
    LDA $EF
    CMP #$04
    BCS LE9A6
    CPY #$02
    BCC LE9A6
    LDX #$5A
    LDY $EF
    CPY #$02
    BNE LE9A6
    LDX #$7E
    INC $02
LE9A6:
    LDA $EC
    CMP #$04
    BNE LE9CA
    LDX #$72
    INC $02
    LDY $EF
    CPY #$02
    BEQ LE9BA
    LDX #$66
    INC $02
LE9BA:
    CPY #$06
    BNE LE9CA
    LDX #$54
    LDA $ED
    AND #$20
    BNE LE9CA
    LDX #$8A
    DEC $02
LE9CA:
    LDY $08
    LDA $EF
    CMP #$05
    BNE LE9DE
    LDA $ED
    BEQ LE9FA
    AND #$08
    BEQ LEA37
    LDX #$B4
    BNE LE9FA
LE9DE:
    CPX #$48
    BEQ LE9FA
    LDA $0796,Y
    CMP #$05
    BCS LEA37
    CPX #$3C
    BNE LE9FA
    CMP #$01
    BEQ LEA37
    INC $02
    INC $02
    INC $02
    JMP LEA29
LE9FA:
    LDA $EF
    CMP #$06
    BEQ LEA37
    CMP #$08
    BEQ LEA37
    CMP #$0C
    BEQ LEA37
    CMP #$18
    BCS LEA37
    LDY #$00
    CMP #$15
    BNE LEA22
    INY
    LDA $075F
    CMP #$07
    BCS LEA37
    LDX #$A2
    LDA #$03
    STA $EC
    BNE LEA37
LEA22:
    LDA $09
    AND $E876,Y
    BNE LEA37
LEA29:
    LDA $ED
    AND #$A0
    ORA $0747
    BNE LEA37
    TXA
    CLC
    ADC #$06
    TAX
LEA37:
    LDA $ED
    AND #$20
    BEQ LEA4B
    LDA $EF
    CMP #$04
    BCC LEA4B
    LDY #$01
    STY $0109
    DEY
    STY $EC
LEA4B:
    LDY $EB
    JSR LEBAA
    JSR LEBAA
    JSR LEBAA
    LDX $08
    LDY $06E5,X
    LDA $EF
    CMP #$08
    BNE LEA64
LEA61:
    JMP LEB64
LEA64:
    LDA $0109
    BEQ LEAA6
    LDA $0202,Y
    ORA #$80
    INY
    INY
    JSR LE5B5
    DEY
    DEY
    TYA
    TAX
    LDA $EF
    CMP #$05
    BEQ LEA8A
    CMP #$11
    BEQ LEA8A
    CMP #$15
    BCS LEA8A
    TXA
    CLC
    ADC #$08
    TAX
LEA8A:
    LDA $0201,X
    PHA
    LDA $0205,X
    PHA
    LDA $0211,Y
    STA $0201,X
    LDA $0215,Y
    STA $0205,X
    PLA
    STA $0215,Y
    PLA
    STA $0211,Y
LEAA6:
    LDA $036A
    BNE LEA61
    LDA $EF
    LDX $EC
    CMP #$05
    BNE LEAB6
    JMP LEB64
LEAB6:
    CMP #$07
    BEQ LEAD7
    CMP #$0D
    BEQ LEAD7
    CMP #$0C
    BEQ LEAD7
    CMP #$12
    BNE LEACA
    CPX #$05
    BNE LEB12
LEACA:
    CMP #$15
    BNE LEAD3
    LDA #$42
    STA $0216,Y
LEAD3:
    CPX #$02
    BCC LEB12
LEAD7:
    LDA $036A
    BNE LEB12
    LDA $0202,Y
    AND #$A3
    STA $0202,Y
    STA $020A,Y
    STA $0212,Y
    ORA #$40
    CPX #$05
    BNE LEAF2
    ORA #$80
LEAF2:
    STA $0206,Y
    STA $020E,Y
    STA $0216,Y
    CPX #$04
    BNE LEB12
    LDA $020A,Y
    ORA #$80
    STA $020A,Y
    STA $0212,Y
    ORA #$40
    STA $020E,Y
    STA $0216,Y
LEB12:
    LDA $EF
    CMP #$11
    BNE LEB4E
    LDA $0109
    BNE LEB3E
    LDA $0212,Y
    AND #$81
    STA $0212,Y
    LDA $0216,Y
    ORA #$41
    STA $0216,Y
    LDX $078F
    CPX #$10
    BCS LEB64
    STA $020E,Y
    AND #$81
    STA $020A,Y
    BCC LEB64
LEB3E:
    LDA $0202,Y
    AND #$81
    STA $0202,Y
    LDA $0206,Y
    ORA #$41
    STA $0206,Y
LEB4E:
    LDA $EF
    CMP #$18
    BCC LEB64
    LDA #$82
    STA $020A,Y
    STA $0212,Y
    ORA #$40
    STA $020E,Y
    STA $0216,Y
LEB64:
    LDX $08
    LDA $03D1
    LSR A
    LSR A
    LSR A
    PHA
    BCC LEB74
    LDA #$04
    JSR LEBC1
LEB74:
    PLA
    LSR A
    PHA
    BCC LEB7E
    LDA #$00
    JSR LEBC1
LEB7E:
    PLA
    LSR A
    LSR A
    PHA
    BCC LEB89
    LDA #$10
    JSR LEBB7
LEB89:
    PLA
    LSR A
    PHA
    BCC LEB93
    LDA #$08
    JSR LEBB7
LEB93:
    PLA
    LSR A
    BCC LEBA9
    JSR LEBB7
    LDA $16,X
    CMP #$0C
    BEQ LEBA9
    LDA $B6,X
    CMP #$02
    BNE LEBA9
    JSR LC998
LEBA9:
    RTS
LEBAA:
    LDA $E73E,X
    STA $00
    LDA $E73F,X
LEBB2:
    STA $01
    JMP LF282
LEBB7:
    CLC
    ADC $06E5,X
    TAY
    LDA #$F8
    JMP LE5C1
LEBC1:
    CLC
    ADC $06E5,X
    TAY
    JSR LEC4A
    STA $0210,Y
    RTS
    .byte $85, $85, $86, $86
LEBD1:
    LDA $03BC
    STA $02
    LDA $03B1
    STA $05
    LDA #$03
    STA $04
    LSR A
    STA $03
    LDY $06EC,X
    LDX #$00
LEBE7:
    LDA $EBCD,X
    STA $00
    LDA $EBCE,X
    JSR LEBB2
    CPX #$04
    BNE LEBE7
    LDX $08
    LDY $06EC,X
    LDA $074E
    CMP #$01
    BEQ LEC0A
    LDA #$86
    STA $0201,Y
    STA $0205,Y
LEC0A:
    LDA $03E8,X
    CMP #$C4
    BNE LEC35
    LDA #$87
    INY
    JSR LE5BB
    DEY
    LDA #$03
    LDX $074E
    DEX
    BEQ LEC21
    LSR A
LEC21:
    LDX $08
    STA $0202,Y
    ORA #$40
    STA $0206,Y
    ORA #$80
    STA $020E,Y
    AND #$83
    STA $020A,Y
LEC35:
    LDA $03D4
    PHA
    AND #$04
    BEQ LEC45
    LDA #$F8
    STA $0204,Y
    STA $020C,Y
LEC45:
    PLA
LEC46:
    AND #$08
    BEQ LEC52
LEC4A:
    LDA #$F8
    STA $0200,Y
    STA $0208,Y
LEC52:
    RTS
LEC53:
    LDA #$02
    STA $00
    LDA #$75
    LDY $0E
    CPY #$05
    BEQ LEC65
    LDA #$03
    STA $00
    LDA #$84
LEC65:
    LDY $06EC,X
    INY
    JSR LE5BB
    LDA $09
    ASL A
    ASL A
    ASL A
    ASL A
    AND #$C0
    ORA $00
    INY
    JSR LE5BB
    DEY
    DEY
    LDA $03BC
    JSR LE5C1
    LDA $03B1
    STA $0203,Y
    LDA $03F1,X
    SEC
    SBC $071C
    STA $00
    SEC
    SBC $03B1
    ADC $00
    ADC #$06
    STA $0207,Y
    LDA $03BD
    STA $0208,Y
    STA $020C,Y
    LDA $03B2
    STA $020B,Y
    LDA $00
    SEC
    SBC $03B2
    ADC $00
    ADC #$06
    STA $020F,Y
    LDA $03D4
    JSR LEC46
    LDA $03D4
    ASL A
    BCC LECC9
    LDA #$F8
    JSR LE5C1
LECC9:
    LDA $00
    BPL LECDD
    LDA $0203,Y
    CMP $0207,Y
    BCC LECDD
    LDA #$F8
    STA $0204,Y
    STA $020C,Y
LECDD:
    RTS
LECDE:
    LDY $06F1,X
    LDA $03BA
    STA $0200,Y
    LDA $03AF
    STA $0203,Y
LECED:
    LDA $09
    LSR A
    LSR A
    PHA
    AND #$01
    EOR #$64
    STA $0201,Y
    PLA
    LSR A
    LSR A
    LDA #$02
    BCC LED02
    ORA #$C0
LED02:
    STA $0202,Y
    RTS
    .byte $68, $67, $66
LED09:
    LDY $06EC,X
    LDA $24,X
    INC $24,X
    LSR A
    AND #$07
    CMP #$03
    BCS LED61
LED17:
    TAX
    LDA $ED06,X
    INY
    JSR LE5BB
    DEY
    LDX $08
    LDA $03BA
    SEC
    SBC #$04
    STA $0200,Y
    STA $0208,Y
    CLC
    ADC #$08
    STA $0204,Y
    STA $020C,Y
    LDA $03AF
    SEC
    SBC #$04
    STA $0203,Y
    STA $0207,Y
    CLC
    ADC #$08
    STA $020B,Y
    STA $020F,Y
    LDA #$02
    STA $0202,Y
    LDA #$82
    STA $0206,Y
    LDA #$42
    STA $020A,Y
    LDA #$C2
    STA $020E,Y
    RTS
LED61:
    LDA #$00
    STA $24,X
    RTS
LED66:
    LDY $06E5,X
    LDA #$5B
    INY
    JSR LE5B5
    INY
    LDA #$02
    JSR LE5B5
    DEY
    DEY
    LDA $03AE
    STA $0203,Y
    STA $020F,Y
    CLC
    ADC #$08
    STA $0207,Y
    STA $0213,Y
    CLC
    ADC #$08
    STA $020B,Y
    STA $0217,Y
    LDA $CF,X
    TAX
    PHA
    CPX #$20
    BCS LED9C
    LDA #$F8
LED9C:
    JSR LE5BE
    PLA
    CLC
    ADC #$80
    TAX
    CPX #$20
    BCS LEDAA
    LDA #$F8
LEDAA:
    STA $020C,Y
    STA $0210,Y
    STA $0214,Y
    LDA $03D1
    PHA
    AND #$08
    BEQ LEDC3
    LDA #$F8
    STA $0200,Y
    STA $020C,Y
LEDC3:
    PLA
    PHA
    AND #$04
    BEQ LEDD1
    LDA #$F8
    STA $0204,Y
    STA $0210,Y
LEDD1:
    PLA
    AND #$02
    BEQ LEDDE
    LDA #$F8
    STA $0208,Y
    STA $0214,Y
LEDDE:
    LDX $08
    RTS
LEDE1:
    LDY $B5
    DEY
    BNE LEE06
    LDA $03D3
    AND #$08
    BNE LEE06
    LDY $06EE,X
    LDA $03B0
    STA $0203,Y
    LDA $03BB
    STA $0200,Y
    LDA #$74
    STA $0201,Y
    LDA #$02
    STA $0202,Y
LEE06:
    RTS
    .byte $20, $28, $C8, $18, $00, $40, $50, $58, $80, $88, $B8, $78, $60, $A0, $B0, $B8
    .byte $00, $01, $02, $03, $04, $05, $06, $07, $08, $09, $0A, $0B, $0C, $0D, $0E, $0F
    .byte $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $1A, $1B, $1C, $1D, $1E, $1F
    .byte $20, $21, $22, $23, $24, $25, $26, $27, $08, $09, $28, $29, $2A, $2B, $2C, $2D
    .byte $08, $09, $0A, $0B, $0C, $30, $2C, $2D, $08, $09, $0A, $0B, $2E, $2F, $2C, $2D
    .byte $08, $09, $28, $29, $2A, $2B, $5C, $5D, $08, $09, $0A, $0B, $0C, $0D, $5E, $5F
    .byte $FC, $FC, $08, $09, $58, $59, $5A, $5A, $08, $09, $28, $29, $2A, $2B, $0E, $0F
    .byte $FC, $FC, $FC, $FC, $32, $33, $34, $35, $FC, $FC, $FC, $FC, $36, $37, $38, $39
    .byte $FC, $FC, $FC, $FC, $3A, $37, $3B, $3C, $FC, $FC, $FC, $FC, $3D, $3E, $3F, $40
    .byte $FC, $FC, $FC, $FC, $32, $41, $42, $43, $FC, $FC, $FC, $FC, $32, $33, $44, $45
    .byte $FC, $FC, $FC, $FC, $32, $33, $44, $47, $FC, $FC, $FC, $FC, $32, $33, $48, $49
    .byte $FC, $FC, $FC, $FC, $32, $33, $90, $91, $FC, $FC, $FC, $FC, $3A, $37, $92, $93
    .byte $FC, $FC, $FC, $FC, $9E, $9E, $9F, $9F, $FC, $FC, $FC, $FC, $3A, $37, $4F, $4F
    .byte $FC, $FC, $00, $01, $4C, $4D, $4E, $4E, $00, $01, $4C, $4D, $4A, $4A, $4B, $4B
    .byte $31, $46
LEEE9:
    LDA $079E
    BEQ LEEF3
    LDA $09
    LSR A
    BCS LEF33
LEEF3:
    LDA $0E
    CMP #$0B
    BEQ LEF40
    LDA $070B
    BNE LEF3A
    LDY $0704
    BEQ LEF34
    LDA $1D
    CMP #$00
    BEQ LEF34
    JSR LEF34
    LDA $09
    AND #$04
    BNE LEF33
    TAX
    LDY $06E4
    LDA $33
    LSR A
    BCS LEF1F
    INY
    INY
    INY
    INY
LEF1F:
    LDA $0754
    BEQ LEF2D
    LDA $0219,Y
    CMP $EEB5
    BEQ LEF33
    INX
LEF2D:
    LDA $EEE7,X
    STA $0219,Y
LEF33:
    RTS
LEF34:
    JSR LEFEC
    JMP LEF45
LEF3A:
    JSR LF0B0
    JMP LEF45
LEF40:
    LDY #$0E
    LDA $EE07,Y
LEF45:
    STA $06D5
    LDA #$04
    JSR LEFBE
    JSR LF0E9
    LDA $0711
    BEQ LEF7A
    LDY #$00
    LDA $0781
    CMP $0711
    STY $0711
    BCS LEF7A
    STA $0711
    LDY #$07
    LDA $EE07,Y
    STA $06D5
    LDY #$04
    LDA $57
    ORA $0C
    BEQ LEF76
    DEY
LEF76:
    TYA
    JSR LEFBE
LEF7A:
    LDA $03D0
    LSR A
    LSR A
    LSR A
    LSR A
    STA $00
    LDX #$03
    LDA $06E4
    CLC
    ADC #$18
    TAY
LEF8C:
    LDA #$F8
    LSR $00
    BCC LEF95
    JSR LE5C1
LEF95:
    TYA
    SEC
    SBC #$08
    TAY
    DEX
    BPL LEF8C
    RTS
    .byte $58, $01, $00, $60, $FF, $04
LEFA4:
    LDX #$05
LEFA6:
    LDA $EF9E,X
    STA $02,X
    DEX
    BPL LEFA6
    LDX #$B8
    LDY #$04
    JSR LEFDC
    LDA $0226
    ORA #$40
    STA $0222
    RTS
LEFBE:
    STA $07
    LDA $03AD
    STA $0755
    STA $05
    LDA $03B8
    STA $02
    LDA $33
    STA $03
    LDA $03C4
    STA $04
    LDX $06D5
    LDY $06E4
LEFDC:
    LDA $EE17,X
    STA $00
    LDA $EE18,X
    JSR LEBB2
    DEC $07
    BNE LEFDC
    RTS
LEFEC:
    LDA $1D
    CMP #$03
    BEQ LF044
    CMP #$02
    BEQ LF034
    CMP #$01
    BNE LF00B
    LDA $0704
    BNE LF050
    LDY #$06
    LDA $0714
    BNE LF028
    LDY #$00
    JMP LF028
LF00B:
    LDY #$06
    LDA $0714
    BNE LF028
    LDY #$02
    LDA $57
    ORA $0C
    BEQ LF028
    LDA $0700
    CMP #$09
    BCC LF03C
    LDA $45
    AND $33
    BNE LF03C
    INY
LF028:
    JSR LF091
    LDA #$00
    STA $070D
    LDA $EE07,Y
    RTS
LF034:
    LDY #$04
    JSR LF091
    JMP LF062
LF03C:
    LDY #$04
    JSR LF091
    JMP LF068
LF044:
    LDY #$05
    LDA $9F
    BEQ LF028
    JSR LF091
    JMP LF06D
LF050:
    LDY #$01
    JSR LF091
    LDA $0782
    ORA $070D
    BNE LF068
    LDA $0A
    ASL A
    BCS LF068
LF062:
    LDA $070D
    JMP LF0D0
LF068:
    LDA #$03
    JMP LF06F
LF06D:
    LDA #$02
LF06F:
    STA $00
    JSR LF062
    PHA
    LDA $0781
    BNE LF08F
    LDA $070C
    STA $0781
    LDA $070D
    CLC
    ADC #$01
    CMP $00
    BCC LF08C
    LDA #$00
LF08C:
    STA $070D
LF08F:
    PLA
    RTS
LF091:
    LDA $0754
    BEQ LF09B
    TYA
    CLC
    ADC #$08
    TAY
LF09B:
    RTS
    .byte $00, $01, $00, $01, $00, $01, $02, $00, $01, $02, $02, $00, $02, $00, $02, $00
    .byte $02, $00, $02, $00
LF0B0:
    LDY $070D
    LDA $09
    AND #$03
    BNE LF0C6
    INY
    CPY #$0A
    BCC LF0C3
    LDY #$00
    STY $070B
LF0C3:
    STY $070D
LF0C6:
    LDA $0754
    BNE LF0D7
    LDA $F09C,Y
    LDY #$0F
LF0D0:
    ASL A
    ASL A
    ASL A
    ADC $EE07,Y
    RTS
LF0D7:
    TYA
    CLC
    ADC #$0A
    TAX
    LDY #$09
    LDA $F09C,X
    BNE LF0E5
    LDY #$01
LF0E5:
    LDA $EE07,Y
    RTS
LF0E9:
    LDY $06E4
    LDA $0E
    CMP #$0B
    BEQ LF105
    LDA $06D5
    CMP #$50
    BEQ LF117
    CMP #$B8
    BEQ LF117
    CMP #$C0
    BEQ LF117
    CMP #$C8
    BNE LF129
LF105:
    LDA $0212,Y
    AND #$3F
    STA $0212,Y
    LDA $0216,Y
    AND #$3F
    ORA #$40
    STA $0216,Y
LF117:
    LDA $021A,Y
    AND #$3F
    STA $021A,Y
    LDA $021E,Y
    AND #$3F
    ORA #$40
    STA $021E,Y
LF129:
    RTS
LF12A:
    LDX #$00
    LDY #$00
    JMP LF142
LF131:
    LDY #$01
    JSR LF1A8
    LDY #$03
    JMP LF142
LF13B:
    LDY #$00
    JSR LF1A8
    LDY #$02
LF142:
    JSR LF171
    LDX $08
    RTS
LF148:
    LDY #$02
    JSR LF1A8
    LDY #$06
    JMP LF142
LF152:
    LDA #$01
    LDY #$01
    JMP LF165
LF159:
    LDA #$09
    LDY #$04
    JSR LF165
    INX
    INX
    LDA #$09
    INY
LF165:
    STX $00
    CLC
    ADC $00
    TAX
    JSR LF171
    LDX $08
    RTS
LF171:
    LDA $CE,X
    STA $03B8,Y
    LDA $86,X
    SEC
    SBC $071C
    STA $03AD,Y
    RTS
LF180:
    LDX #$00
    LDY #$00
    JMP LF1C0
LF187:
    LDY #$00
    JSR LF1A8
    LDY #$02
    JMP LF1C0
LF191:
    LDY #$01
    JSR LF1A8
    LDY #$03
    JMP LF1C0
LF19B:
    LDY #$02
    JSR LF1A8
    LDY #$06
    JMP LF1C0
    .byte $07, $16, $0D
LF1A8:
    TXA
    CLC
    ADC $F1A5,Y
    TAX
    RTS
LF1AF:
    LDA #$01
    LDY #$01
    JMP LF1BA
LF1B6:
    LDA #$09
    LDY #$04
LF1BA:
    STX $00
    CLC
    ADC $00
    TAX
LF1C0:
    TYA
    PHA
    JSR LF1D7
    ASL A
    ASL A
    ASL A
    ASL A
    ORA $00
    STA $00
    PLA
    TAY
    LDA $00
    STA $03D0,Y
    LDX $08
    RTS
LF1D7:
    JSR LF1F6
    LSR A
    LSR A
    LSR A
    LSR A
    STA $00
    JMP LF239
    .byte $7F, $3F, $1F, $0F, $07, $03, $01, $00, $80, $C0, $E0, $F0, $F8, $FC, $FE, $FF
    .byte $07, $0F, $07
LF1F6:
    STX $04
    LDY #$01
LF1FA:
    LDA $071C,Y
    SEC
    SBC $86,X
    STA $07
    LDA $071A,Y
    SBC $6D,X
    LDX $F1F3,Y
    CMP #$00
    BMI LF21E
    LDX $F1F4,Y
    CMP #$01
    BPL LF21E
    LDA #$38
    STA $06
    LDA #$08
    JSR LF26D
LF21E:
    LDA $F1E3,X
    LDX $04
    CMP #$00
    BNE LF22A
    DEY
    BPL LF1FA
LF22A:
    RTS
    .byte $00, $08, $0C, $0E, $0F, $07, $03, $01, $00, $04, $00, $04, $FF, $00
LF239:
    STX $04
    LDY #$01
LF23D:
    LDA $F237,Y
    SEC
    SBC $CE,X
    STA $07
    LDA #$01
    SBC $B5,X
    LDX $F234,Y
    CMP #$00
    BMI LF260
    LDX $F235,Y
    CMP #$01
    BPL LF260
    LDA #$20
    STA $06
    LDA #$04
    JSR LF26D
LF260:
    LDA $F22B,X
    LDX $04
    CMP #$00
    BNE LF26C
    DEY
    BPL LF23D
LF26C:
    RTS
LF26D:
    STA $05
    LDA $07
    CMP $06
    BCS LF281
    LSR A
    LSR A
    LSR A
    AND #$07
    CPY #$01
    BCS LF280
    ADC $05
LF280:
    TAX
LF281:
    RTS
LF282:
    LDA $03
    LSR A
    LSR A
    LDA $00
    BCC LF296
    STA $0205,Y
    LDA $01
    STA $0201,Y
    LDA #$40
    BNE LF2A0
LF296:
    STA $0201,Y
    LDA $01
    STA $0205,Y
    LDA #$00
LF2A0:
    ORA $04
    STA $0202,Y
    STA $0206,Y
    LDA $02
    STA $0200,Y
    STA $0204,Y
    LDA $05
    STA $0203,Y
    CLC
    ADC #$08
    STA $0207,Y
    LDA $02
    CLC
    ADC #$08
    STA $02
    TYA
    CLC
    ADC #$08
    TAY
    INX
    INX
    RTS
    .byte $FF, $FF, $FF, $FF, $FF, $FF
LF2D0:
    LDA $0770
    BNE LF2D9
    STA $4015
    RTS
LF2D9:
    LDA #$FF
    STA $4017
    LDA #$0F
    STA $4015
    LDA $07C6
    BNE LF2EE
    LDA $FA
    CMP #$01
    BNE LF34B
LF2EE:
    LDA $07B2
    BNE LF316
    LDA $FA
    BEQ LF35D
    STA $07B2
    STA $07C6
    LDA #$00
    STA $4015
    STA $F1
    STA $F2
    STA $F3
    LDA #$0F
    STA $4015
    LDA #$2A
    STA $07BB
LF312:
    LDA #$44
    BNE LF327
LF316:
    LDA $07BB
    CMP #$24
    BEQ LF325
    CMP #$1E
    BEQ LF312
    CMP #$18
    BNE LF32E
LF325:
    LDA #$64
LF327:
    LDX #$84
    LDY #$7F
    JSR LF388
LF32E:
    DEC $07BB
    BNE LF35D
    LDA #$00
    STA $4015
    LDA $07B2
    CMP #$02
    BNE LF344
    LDA #$00
    STA $07C6
LF344:
    LDA #$00
    STA $07B2
    BEQ LF35D
LF34B:
    JSR LF41B
    JSR LF57C
    JSR LF667
    JSR LF694
    LDA #$00
    STA $FB
    STA $FC
LF35D:
    LDA #$00
    STA $FF
    STA $FE
    STA $FD
    STA $FA
    LDY $07C0
    LDA $F4
    AND #$03
    BEQ LF377
    INC $07C0
    CPY #$30
    BCC LF37D
LF377:
    TYA
    BEQ LF37D
    DEC $07C0
LF37D:
    STY $4011
    RTS
LF381:
    STY $4001
    STX $4000
    RTS
LF388:
    JSR LF381
LF38B:
    LDX #$00
LF38D:
    TAY
    LDA $FF01,Y
    BEQ LF39E
    STA $4002,X
    LDA $FF00,Y
    ORA #$08
    STA $4003,X
LF39E:
    RTS
LF39F:
    STX $4004
    STY $4005
    RTS
LF3A6:
    JSR LF39F
LF3A9:
    LDX #$04
    BNE LF38D
LF3AD:
    LDX #$08
    BNE LF38D
    .byte $9F, $9B, $98, $96, $95, $94, $92, $90, $90, $9A, $97, $95, $93, $92
LF3BF:
    LDA #$40
    STA $07BB
    LDA #$62
    JSR LF38B
    LDX #$99
    BNE LF3F2
LF3CD:
    LDA #$26
    BNE LF3D3
LF3D1:
    LDA #$18
LF3D3:
    LDX #$82
    LDY #$A7
    JSR LF388
    LDA #$28
    STA $07BB
LF3DF:
    LDA $07BB
    CMP #$25
    BNE LF3EC
    LDX #$5F
    LDY #$F6
    BNE LF3F4
LF3EC:
    CMP #$20
    BNE LF419
    LDX #$48
LF3F2:
    LDY #$BC
LF3F4:
    JSR LF381
    BNE LF419
LF3F9:
    LDA #$05
    LDY #$99
    BNE LF403
LF3FF:
    LDA #$0A
    LDY #$93
LF403:
    LDX #$9E
    STA $07BB
    LDA #$0C
    JSR LF388
LF40D:
    LDA $07BB
    CMP #$06
    BNE LF419
    LDA #$BB
    STA $4001
LF419:
    BNE LF47B
LF41B:
    LDY $FF
    BEQ LF43F
    STY $F1
    BMI LF3CD
    LSR $FF
    BCS LF3D1
    LSR $FF
    BCS LF3FF
    LSR $FF
    BCS LF45B
    LSR $FF
    BCS LF47D
    LSR $FF
    BCS LF4B6
    LSR $FF
    BCS LF3F9
    LSR $FF
    BCS LF3BF
LF43F:
    LDA $F1
    BEQ LF45A
    BMI LF3DF
    LSR A
    BCS LF3DF
    LSR A
    BCS LF40D
    LSR A
    BCS LF469
    LSR A
    BCS LF48D
    LSR A
    BCS LF4BB
    LSR A
    BCS LF40D
    LSR A
    BCS LF4A2
LF45A:
    RTS
LF45B:
    LDA #$0E
    STA $07BB
    LDY #$9C
    LDX #$9E
    LDA #$26
    JSR LF388
LF469:
    LDY $07BB
    LDA $F3B0,Y
    STA $4000
    CPY #$06
    BNE LF47B
    LDA #$9E
    STA $4002
LF47B:
    BNE LF4A2
LF47D:
    LDA #$0E
    LDY #$CB
    LDX #$9F
    STA $07BB
    LDA #$28
    JSR LF388
    BNE LF4A2
LF48D:
    LDY $07BB
    CPY #$08
    BNE LF49D
    LDA #$A0
    STA $4002
    LDA #$9F
    BNE LF49F
LF49D:
    LDA #$90
LF49F:
    STA $4000
LF4A2:
    DEC $07BB
    BNE LF4B5
LF4A7:
    LDX #$00
    STX $F1
    LDX #$0E
    STX $4015
    LDX #$0F
    STX $4015
LF4B5:
    RTS
LF4B6:
    LDA #$2F
    STA $07BB
LF4BB:
    LDA $07BB
    LSR A
    BCS LF4D1
    LSR A
    BCS LF4D1
    AND #$02
    BEQ LF4D1
    LDY #$91
    LDX #$9A
    LDA #$44
    JSR LF388
LF4D1:
    JMP LF4A2
    .byte $58, $02, $54, $56, $4E, $44, $4C, $52, $4C, $48, $3E, $36, $3E, $36, $30, $28
    .byte $4A, $50, $4A, $64, $3C, $32, $3C, $32, $2C, $24, $3A, $64, $3A, $34, $2C, $22
    .byte $2C, $22, $1C, $14, $14, $04, $22, $24, $16, $04, $24, $26, $18, $04, $26, $28
    .byte $1A, $04, $28, $2A, $1C, $04, $2A, $2C, $1E, $04, $2C, $2E, $20, $04, $2E, $30
    .byte $22, $04, $30, $32
LF518:
    LDA #$35
    LDX #$8D
    BNE LF522
LF51E:
    LDA #$06
    LDX #$98
LF522:
    STA $07BD
    LDY #$7F
    LDA #$42
    JSR LF3A6
LF52C:
    LDA $07BD
    CMP #$30
    BNE LF538
    LDA #$54
    STA $4006
LF538:
    BNE LF568
LF53A:
    LDA #$20
    STA $07BD
    LDY #$94
    LDA #$5E
    BNE LF550
LF545:
    LDA $07BD
    CMP #$18
    BNE LF568
    LDY #$93
    LDA #$18
LF550:
    BNE LF5D1
LF552:
    LDA #$36
    STA $07BD
LF557:
    LDA $07BD
    LSR A
    BCS LF568
    TAY
    LDA $F4D9,Y
    LDX #$5D
    LDY #$7F
LF565:
    JSR LF3A6
LF568:
    DEC $07BD
    BNE LF57B
LF56D:
    LDX #$00
    STX $F2
LF571:
    LDX #$0D
    STX $4015
    LDX #$0F
    STX $4015
LF57B:
    RTS
LF57C:
    LDA $F2
    AND #$40
    BNE LF5E7
    LDY $FE
    BEQ LF5A6
    STY $F2
    BMI LF5C8
    LSR $FE
    BCS LF518
    LSR $FE
    BCS LF5FC
    LSR $FE
    BCS LF600
    LSR $FE
    BCS LF53A
    LSR $FE
    BCS LF51E
    LSR $FE
    BCS LF552
    LSR $FE
    BCS LF5E2
LF5A6:
    LDA $F2
    BEQ LF5C1
    BMI LF5D3
    LSR A
    BCS LF5C2
    LSR A
    BCS LF60F
    LSR A
    BCS LF60F
    LSR A
    BCS LF545
    LSR A
    BCS LF5C2
    LSR A
    BCS LF557
    LSR A
    BCS LF5E7
LF5C1:
    RTS
LF5C2:
    JMP LF52C
LF5C5:
    JMP LF568
LF5C8:
    LDA #$38
    STA $07BD
    LDY #$C4
    LDA #$18
LF5D1:
    BNE LF5DE
LF5D3:
    LDA $07BD
    CMP #$08
    BNE LF568
    LDY #$A4
    LDA #$5A
LF5DE:
    LDX #$9F
LF5E0:
    BNE LF565
LF5E2:
    LDA #$30
    STA $07BD
LF5E7:
    LDA $07BD
    LDX #$03
LF5EC:
    LSR A
    BCS LF5C5
    DEX
    BNE LF5EC
    TAY
    LDA $F4D3,Y
    LDX #$82
    LDY #$7F
    BNE LF5E0
LF5FC:
    LDA #$10
    BNE LF602
LF600:
    LDA #$20
LF602:
    STA $07BD
    LDA #$7F
    STA $4005
    LDA #$00
    STA $07BE
LF60F:
    INC $07BE
    LDA $07BE
    LSR A
    TAY
    CPY $07BD
    BEQ LF628
    LDA #$9D
    STA $4004
    LDA $F4F8,Y
    JSR LF3A9
    RTS
LF628:
    JMP LF56D
    .byte $01, $0E, $0E, $0D, $0B, $06, $0C, $0F, $0A, $09, $03, $0D, $08, $0D, $06, $0C
LF63B:
    LDA #$20
    STA $07BF
LF640:
    LDA $07BF
    LSR A
    BCC LF658
    TAY
    LDX $F62B,Y
    LDA $FFEA,Y
LF64D:
    STA $400C
    STX $400E
    LDA #$18
    STA $400F
LF658:
    DEC $07BF
    BNE LF666
    LDA #$F0
    STA $400C
    LDA #$00
    STA $F3
LF666:
    RTS
LF667:
    LDY $FD
    BEQ LF675
    STY $F3
    LSR $FD
    BCS LF63B
    LSR $FD
    BCS LF680
LF675:
    LDA $F3
    BEQ LF67F
    LSR A
    BCS LF640
    LSR A
    BCS LF685
LF67F:
    RTS
LF680:
    LDA #$40
    STA $07BF
LF685:
    LDA $07BF
    LSR A
    TAY
    LDX #$0F
    LDA $FFC9,Y
    BNE LF64D
LF691:
    JMP LF73A
LF694:
    LDA $FC
    BNE LF6A4
    LDA $FB
    BNE LF6C8
    LDA $07B1
    ORA $F4
    BNE LF691
    RTS
LF6A4:
    STA $07B1
    CMP #$01
    BNE LF6B1
    JSR LF4A7
    JSR LF571
LF6B1:
    LDX $F4
    STX $07C5
    LDY #$00
    STY $07C4
    STY $F4
    CMP #$40
    BNE LF6F1
    LDX #$08
    STX $07C4
    BNE LF6F1
LF6C8:
    CMP #$04
    BNE LF6CF
    JSR LF4A7
LF6CF:
    LDY #$10
LF6D1:
    STY $07C7
LF6D4:
    LDY #$00
    STY $07B1
    STA $F4
    CMP #$01
    BNE LF6ED
    INC $07C7
    LDY $07C7
    CPY #$32
    BNE LF6F5
    LDY #$11
    BNE LF6D1
LF6ED:
    LDY #$08
    STY $F7
LF6F1:
    INY
    LSR A
    BCC LF6F1
LF6F5:
    LDA $F90C,Y
    TAY
    LDA $F90D,Y
    STA $F0
    LDA $F90E,Y
    STA $F5
    LDA $F90F,Y
    STA $F6
    LDA $F910,Y
    STA $F9
    LDA $F911,Y
    STA $F8
    LDA $F912,Y
    STA $07B0
    STA $07C1
    LDA #$01
    STA $07B4
    STA $07B6
    STA $07B9
    STA $07BA
    LDA #$00
    STA $F7
    STA $07CA
    LDA #$0B
    STA $4015
    LDA #$0F
    STA $4015
LF73A:
    DEC $07B4
    BNE LF79E
    LDY $F7
    INC $F7
    LDA ($F5),Y
    BEQ LF74B
    BPL LF786
    BNE LF77A
LF74B:
    LDA $07B1
    CMP #$40
    BNE LF757
    LDA $07C5
    BNE LF774
LF757:
    AND #$04
    BNE LF777
    LDA $F4
    AND #$5F
    BNE LF774
    LDA #$00
    STA $F4
    STA $07B1
    STA $4008
    LDA #$90
    STA $4000
    STA $4004
    RTS
LF774:
    JMP LF6D4
LF777:
    JMP LF6A4
LF77A:
    JSR LF8CB
    STA $07B3
    LDY $F7
    INC $F7
    LDA ($F5),Y
LF786:
    LDX $F2
    BNE LF798
    JSR LF3A9
    BEQ LF792
    JSR LF8D8
LF792:
    STA $07B5
    JSR LF39F
LF798:
    LDA $07B3
    STA $07B4
LF79E:
    LDA $F2
    BNE LF7BC
    LDA $07B1
    AND #$91
    BNE LF7BC
    LDY $07B5
    BEQ LF7B1
    DEC $07B5
LF7B1:
    JSR LF8F4
    STA $4004
    LDX #$7F
    STX $4005
LF7BC:
    LDY $F8
    BEQ LF81A
    DEC $07B6
    BNE LF7F7
LF7C5:
    LDY $F8
    INC $F8
    LDA ($F5),Y
    BNE LF7DC
    LDA #$83
    STA $4000
    LDA #$94
    STA $4001
    STA $07CA
    BNE LF7C5
LF7DC:
    JSR LF8C5
    STA $07B6
    LDY $F1
    BNE LF81A
    TXA
    AND #$3E
    JSR LF38B
    BEQ LF7F1
    JSR LF8D8
LF7F1:
    STA $07B7
    JSR LF381
LF7F7:
    LDA $F1
    BNE LF81A
    LDA $07B1
    AND #$91
    BNE LF810
    LDY $07B7
    BEQ LF80A
    DEC $07B7
LF80A:
    JSR LF8F4
    STA $4000
LF810:
    LDA $07CA
    BNE LF817
    LDA #$7F
LF817:
    STA $4001
LF81A:
    LDA $F9
    DEC $07B9
    BNE LF86D
    LDY $F9
    INC $F9
    LDA ($F5),Y
    BEQ LF86A
    BPL LF83E
    JSR LF8CB
    STA $07B8
    LDA #$1F
    STA $4008
    LDY $F9
    INC $F9
    LDA ($F5),Y
    BEQ LF86A
LF83E:
    JSR LF3AD
    LDX $07B8
    STX $07B9
    LDA $07B1
    AND #$6E
    BNE LF854
    LDA $F4
    AND #$0A
    BEQ LF86D
LF854:
    TXA
    CMP #$12
    BCS LF868
    LDA $07B1
    AND #$08
    BEQ LF864
    LDA #$0F
    BNE LF86A
LF864:
    LDA #$1F
    BNE LF86A
LF868:
    LDA #$FF
LF86A:
    STA $4008
LF86D:
    LDA $F4
    AND #$F3
    BEQ LF8C4
    DEC $07BA
    BNE LF8C4
LF878:
    LDY $07B0
    INC $07B0
    LDA ($F5),Y
    BNE LF88A
    LDA $07C1
    STA $07B0
    BNE LF878
LF88A:
    JSR LF8C5
    STA $07BA
    TXA
    AND #$3E
    BEQ LF8B9
    CMP #$30
    BEQ LF8B1
    CMP #$20
    BEQ LF8A9
    AND #$10
    BEQ LF8B9
    LDA #$1C
    LDX #$03
    LDY #$18
    BNE LF8BB
LF8A9:
    LDA #$1C
    LDX #$0C
    LDY #$18
    BNE LF8BB
LF8B1:
    LDA #$1C
    LDX #$03
    LDY #$58
    BNE LF8BB
LF8B9:
    LDA #$10
LF8BB:
    STA $400C
    STX $400E
    STY $400F
LF8C4:
    RTS
LF8C5:
    TAX
    ROR A
    TXA
    ROL A
    ROL A
    ROL A
LF8CB:
    AND #$07
    CLC
    ADC $F0
    ADC $07C4
    TAY
    LDA $FF66,Y
    RTS
LF8D8:
    LDA $07B1
    AND #$08
    BEQ LF8E3
    LDA #$04
    BNE LF8EF
LF8E3:
    LDA $F4
    AND #$7D
    BEQ LF8ED
    LDA #$08
    BNE LF8EF
LF8ED:
    LDA #$28
LF8EF:
    LDX #$82
    LDY #$7F
    RTS
LF8F4:
    LDA $07B1
    AND #$08
    BEQ LF8FF
    LDA $FF96,Y
    RTS
LF8FF:
    LDA $F4
    AND #$7D
    BEQ LF909
    LDA $FF9A,Y
    RTS
LF909:
    LDA $FFA2,Y
    RTS
    .byte $A5, $59, $54, $64, $59, $3C, $31, $4B, $69, $5E, $46, $4F, $36, $8D, $36, $4B
    .byte $8D, $69, $69, $6F, $75, $6F, $7B, $6F, $75, $6F, $7B, $81, $87, $81, $8D, $69
    .byte $69, $93, $99, $93, $9F, $93, $99, $93, $9F, $81, $87, $81, $8D, $93, $99, $93
    .byte $9F, $08, $72, $FC, $27, $18, $20, $B8, $F9, $2E, $1A, $40, $20, $B0, $FC, $3D
    .byte $21, $20, $C4, $FC, $3F, $1D, $18, $11, $FD, $00, $00, $08, $1C, $FA, $00, $00
    .byte $A4, $FB, $93, $62, $10, $C8, $FE, $24, $14, $18, $45, $FC, $1E, $14, $08, $52
    .byte $FD, $A0, $70, $68, $08, $51, $FE, $4C, $24, $18, $01, $FA, $2D, $1C, $B8, $18
    .byte $49, $FA, $20, $12, $70, $18, $75, $FA, $1B, $10, $44, $18, $9D, $FA, $11, $0A
    .byte $1C, $18, $C2, $FA, $2D, $10, $58, $18, $DB, $FA, $14, $0D, $3F, $18, $F9, $FA
    .byte $15, $0D, $21, $18, $25, $FB, $18, $10, $7A, $18, $4B, $FB, $19, $0F, $54, $18
    .byte $74, $FB, $1E, $12, $2B, $18, $72, $FB, $1E, $0F, $2D, $84, $2C, $2C, $2C, $82
    .byte $04, $2C, $04, $85, $2C, $84, $2C, $2C, $2A, $2A, $2A, $82, $04, $2A, $04, $85
    .byte $2A, $84, $2A, $2A, $00, $1F, $1F, $1F, $98, $1F, $1F, $98, $9E, $98, $1F, $1D
    .byte $1D, $1D, $94, $1D, $1D, $94, $9C, $94, $1D, $86, $18, $85, $26, $30, $84, $04
    .byte $26, $30, $86, $14, $85, $22, $2C, $84, $04, $22, $2C, $21, $D0, $C4, $D0, $31
    .byte $D0, $C4, $D0, $00, $85, $2C, $22, $1C, $84, $26, $2A, $82, $28, $26, $04, $87
    .byte $22, $34, $3A, $82, $40, $04, $36, $84, $3A, $34, $82, $2C, $30, $85, $2A, $00
    .byte $5D, $55, $4D, $15, $19, $96, $15, $D5, $E3, $EB, $2D, $A6, $2B, $27, $9C, $9E
    .byte $59, $85, $22, $1C, $14, $84, $1E, $22, $82, $20, $1E, $04, $87, $1C, $2C, $34
    .byte $82, $36, $04, $30, $34, $04, $2C, $04, $26, $2A, $85, $22, $84, $04, $82, $3A
    .byte $38, $36, $32, $04, $34, $04, $24, $26, $2C, $04, $26, $2C, $30, $00, $05, $B4
    .byte $B2, $B0, $2B, $AC, $84, $9C, $9E, $A2, $84, $94, $9C, $9E, $85, $14, $22, $84
    .byte $2C, $85, $1E, $82, $2C, $84, $2C, $1E, $84, $04, $82, $3A, $38, $36, $32, $04
    .byte $34, $04, $64, $04, $64, $86, $64, $00, $05, $B4, $B2, $B0, $2B, $AC, $84, $37
    .byte $B6, $B6, $45, $85, $14, $1C, $82, $22, $84, $2C, $4E, $82, $4E, $84, $4E, $22
    .byte $84, $04, $85, $32, $85, $30, $86, $2C, $04, $00, $05, $A4, $05, $9E, $05, $9D
    .byte $85, $84, $14, $85, $24, $28, $2C, $82, $22, $84, $22, $14, $21, $D0, $C4, $D0
    .byte $31, $D0, $C4, $D0, $00, $82, $2C, $84, $2C, $2C, $82, $2C, $30, $04, $34, $2C
    .byte $04, $26, $86, $22, $00, $A4, $25, $25, $A4, $29, $A2, $1D, $9C, $95, $82, $2C
    .byte $2C, $04, $2C, $04, $2C, $30, $85, $34, $04, $04, $00, $A4, $25, $25, $A4, $A8
    .byte $63, $04, $85, $0E, $1A, $84, $24, $85, $22, $14, $84, $0C, $82, $34, $84, $34
    .byte $34, $82, $2C, $84, $34, $86, $3A, $04, $00, $A0, $21, $21, $A0, $21, $2B, $05
    .byte $A3, $82, $18, $84, $18, $18, $82, $18, $18, $04, $86, $3A, $22, $31, $90, $31
    .byte $90, $31, $71, $31, $90, $90, $90, $00, $82, $34, $84, $2C, $85, $22, $84, $24
    .byte $82, $26, $36, $04, $36, $86, $26, $00, $AC, $27, $5D, $1D, $9E, $2D, $AC, $9F
    .byte $85, $14, $82, $20, $84, $22, $2C, $1E, $1E, $82, $2C, $2C, $1E, $04, $87, $2A
    .byte $40, $40, $40, $3A, $36, $82, $34, $2C, $04, $26, $86, $22, $00, $E3, $F7, $F7
    .byte $F7, $F5, $F1, $AC, $27, $9E, $9D, $85, $18, $82, $1E, $84, $22, $2A, $22, $22
    .byte $82, $2C, $2C, $22, $04, $86, $04, $82, $2A, $36, $04, $36, $87, $36, $34, $30
    .byte $86, $2C, $04, $00, $00, $68, $6A, $6C, $45, $A2, $31, $B0, $F1, $ED, $EB, $A2
    .byte $1D, $9C, $95, $86, $04, $85, $22, $82, $22, $87, $22, $26, $2A, $84, $2C, $22
    .byte $86, $14, $51, $90, $31, $11, $00, $80, $22, $28, $22, $26, $22, $24, $22, $26
    .byte $22, $28, $22, $2A, $22, $28, $22, $26, $22, $28, $22, $26, $22, $24, $22, $26
    .byte $22, $28, $22, $2A, $22, $28, $22, $26, $20, $26, $20, $24, $20, $26, $20, $28
    .byte $20, $26, $20, $28, $20, $26, $20, $24, $20, $26, $20, $24, $20, $26, $20, $28
    .byte $20, $26, $20, $28, $20, $26, $20, $24, $28, $30, $28, $32, $28, $30, $28, $2E
    .byte $28, $30, $28, $2E, $28, $2C, $28, $2E, $28, $30, $28, $32, $28, $30, $28, $2E
    .byte $28, $30, $28, $2E, $28, $2C, $28, $2E, $00, $04, $70, $6E, $6C, $6E, $70, $72
    .byte $70, $6E, $70, $6E, $6C, $6E, $70, $72, $70, $6E, $6E, $6C, $6E, $70, $6E, $70
    .byte $6E, $6C, $6E, $6C, $6E, $70, $6E, $70, $6E, $6C, $76, $78, $76, $74, $76, $74
    .byte $72, $74, $76, $78, $76, $74, $76, $74, $72, $74, $84, $1A, $83, $18, $20, $84
    .byte $1E, $83, $1C, $28, $26, $1C, $1A, $1C, $82, $2C, $04, $04, $22, $04, $04, $84
    .byte $1C, $87, $26, $2A, $26, $84, $24, $28, $24, $80, $22, $00, $9C, $05, $94, $05
    .byte $0D, $9F, $1E, $9C, $98, $9D, $82, $22, $04, $04, $1C, $04, $04, $84, $14, $86
    .byte $1E, $80, $16, $80, $14, $81, $1C, $30, $04, $30, $30, $04, $1E, $32, $04, $32
    .byte $32, $04, $20, $34, $04, $34, $34, $04, $36, $04, $84, $36, $00, $46, $A4, $64
    .byte $A4, $48, $A6, $66, $A6, $4A, $A8, $68, $A8, $6A, $44, $2B, $81, $2A, $42, $04
    .byte $42, $42, $04, $2C, $64, $04, $64, $64, $04, $2E, $46, $04, $46, $46, $04, $22
    .byte $04, $84, $22, $87, $04, $06, $0C, $14, $1C, $22, $86, $2C, $22, $87, $04, $60
    .byte $0E, $14, $1A, $24, $86, $2C, $24, $87, $04, $08, $10, $18, $1E, $28, $86, $30
    .byte $30, $80, $64, $00, $CD, $D5, $DD, $E3, $ED, $F5, $BB, $B5, $CF, $D5, $DB, $E5
    .byte $ED, $F3, $BD, $B3, $D1, $D9, $DF, $E9, $F1, $F7, $BF, $FF, $FF, $FF, $34, $00
    .byte $86, $04, $87, $14, $1C, $22, $86, $34, $84, $2C, $04, $04, $04, $87, $14, $1A
    .byte $24, $86, $32, $84, $2C, $04, $86, $04, $87, $18, $1E, $28, $86, $36, $87, $30
    .byte $30, $30, $80, $2C, $82, $14, $2C, $62, $26, $10, $28, $80, $04, $82, $14, $2C
    .byte $62, $26, $10, $28, $80, $04, $82, $08, $1E, $5E, $18, $60, $1A, $80, $04, $82
    .byte $08, $1E, $5E, $18, $60, $1A, $86, $04, $83, $1A, $18, $16, $84, $14, $1A, $18
    .byte $0E, $0C, $16, $83, $14, $20, $1E, $1C, $28, $26, $87, $24, $1A, $12, $10, $62
    .byte $0E, $80, $04, $04, $00, $82, $18, $1C, $20, $22, $26, $28, $81, $2A, $2A, $2A
    .byte $04, $2A, $04, $83, $2A, $82, $22, $86, $34, $32, $34, $81, $04, $22, $26, $2A
    .byte $2C, $30, $86, $34, $83, $32, $82, $36, $84, $34, $85, $04, $81, $22, $86, $30
    .byte $2E, $30, $81, $04, $22, $26, $2A, $2C, $2E, $86, $30, $83, $22, $82, $36, $84
    .byte $34, $85, $04, $81, $22, $86, $3A, $3A, $3A, $82, $3A, $81, $40, $82, $04, $81
    .byte $3A, $86, $36, $36, $36, $82, $36, $81, $3A, $82, $04, $81, $36, $86, $34, $82
    .byte $26, $2A, $36, $81, $34, $34, $85, $34, $81, $2A, $86, $2C, $00, $84, $90, $B0
    .byte $84, $50, $50, $B0, $00, $98, $96, $94, $92, $94, $96, $58, $58, $58, $44, $5C
    .byte $44, $9F, $A3, $A1, $A3, $85, $A3, $E0, $A6, $23, $C4, $9F, $9D, $9F, $85, $9F
    .byte $D2, $A6, $23, $C4, $B5, $B1, $AF, $85, $B1, $AF, $AD, $85, $95, $9E, $A2, $AA
    .byte $6A, $6A, $6B, $5E, $9D, $84, $04, $04, $82, $22, $86, $22, $82, $14, $22, $2C
    .byte $12, $22, $2A, $14, $22, $2C, $1C, $22, $2C, $14, $22, $2C, $12, $22, $2A, $14
    .byte $22, $2C, $1C, $22, $2C, $18, $22, $2A, $16, $20, $28, $18, $22, $2A, $12, $22
    .byte $2A, $18, $22, $2A, $12, $22, $2A, $14, $22, $2C, $0C, $22, $2C, $14, $22, $34
    .byte $12, $22, $30, $10, $22, $2E, $16, $22, $34, $18, $26, $36, $16, $26, $36, $14
    .byte $26, $36, $12, $22, $36, $5C, $22, $34, $0C, $22, $22, $81, $1E, $1E, $85, $1E
    .byte $81, $12, $86, $14, $81, $2C, $22, $1C, $2C, $22, $1C, $85, $2C, $04, $81, $2E
    .byte $24, $1E, $2E, $24, $1E, $85, $2E, $04, $81, $32, $28, $22, $32, $28, $22, $85
    .byte $32, $87, $36, $36, $36, $84, $3A, $00, $5C, $54, $4C, $5C, $54, $4C, $5C, $1C
    .byte $1C, $5C, $5C, $5C, $5C, $5E, $56, $4E, $5E, $56, $4E, $5E, $1E, $1E, $5E, $5E
    .byte $5E, $5E, $62, $5A, $50, $62, $5A, $50, $62, $22, $22, $62, $E7, $E7, $E7, $2B
    .byte $86, $14, $81, $14, $80, $14, $14, $81, $14, $14, $14, $14, $86, $16, $81, $16
    .byte $80, $16, $16, $81, $16, $16, $16, $16, $81, $28, $22, $1A, $28, $22, $1A, $28
    .byte $80, $28, $28, $81, $28, $87, $2C, $2C, $2C, $84, $30, $83, $04, $84, $0C, $83
    .byte $62, $10, $84, $12, $83, $1C, $22, $1E, $22, $26, $18, $1E, $04, $1C, $00, $E3
    .byte $E1, $E3, $1D, $DE, $E0, $23, $EC, $75, $74, $F0, $F4, $F6, $EA, $31, $2D, $83
    .byte $12, $14, $04, $18, $1A, $1C, $14, $26, $22, $1E, $1C, $18, $1E, $22, $0C, $14
    .byte $FF, $FF, $FF, $00, $88, $00, $2F, $00, $00, $02, $A6, $02, $80, $02, $5C, $02
    .byte $3A, $02, $1A, $01, $DF, $01, $C4, $01, $AB, $01, $93, $01, $7C, $01, $67, $01
    .byte $53, $01, $40, $01, $2E, $01, $1D, $01, $0D, $00, $FE, $00, $EF, $00, $E2, $00
    .byte $D5, $00, $C9, $00, $BE, $00, $B3, $00, $A9, $00, $A0, $00, $97, $00, $8E, $00
    .byte $86, $00, $77, $00, $7E, $00, $71, $00, $54, $00, $64, $00, $5F, $00, $59, $00
    .byte $50, $00, $47, $00, $43, $00, $3B, $00, $35, $00, $2A, $00, $23, $04, $75, $03
    .byte $57, $02, $F9, $02, $CF, $01, $FC, $00, $6A, $05, $0A, $14, $28, $50, $1E, $3C
    .byte $02, $04, $08, $10, $20, $40, $18, $30, $0C, $03, $06, $0C, $18, $30, $12, $24
    .byte $08, $36, $03, $09, $06, $12, $1B, $24, $0C, $24, $02, $06, $04, $0C, $12, $18
    .byte $08, $12, $01, $03, $02, $06, $09, $0C, $04, $98, $99, $9A, $9B, $90, $94, $94
    .byte $95, $95, $96, $97, $98, $90, $91, $92, $92, $93, $93, $93, $94, $94, $94, $94
    .byte $94, $94, $95, $95, $95, $95, $95, $95, $96, $96, $96, $96, $96, $96, $96, $96
    .byte $96, $96, $96, $96, $96, $96, $96, $96, $96, $95, $95, $94, $93, $15, $16, $16
    .byte $17, $17, $18, $19, $19, $1A, $1A, $1C, $1D, $1D, $1E, $1E, $1F, $1F, $1F, $1F
    .byte $1E, $1D, $1C, $1E, $1F, $1F, $1E, $1D, $1C, $1A, $18, $16, $14, $15, $16, $16
    .byte $17, $17, $18
IRQ:
    ORA $1A19,Y
    .byte $1A, $1C, $1D, $1D, $1E, $1E, $1F
    .word NMI
    .word Reset
    .word IRQ
