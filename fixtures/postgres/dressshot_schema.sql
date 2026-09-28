create table public."User" (
    id text primary key
);

create table public."SellerProfile" (
    id text primary key,
    "userId" text constraint "SellerProfile_userId_fkey" references public."User"(id)
);

create table public."ClothingSession" (
    id text primary key,
    "sellerProfileId" text not null,
    constraint "ClothingSession_sellerProfileId_fkey"
        foreign key ("sellerProfileId") references public."SellerProfile"(id)
);

create table public."ClothingItem" (
    id text primary key,
    "clothingSessionId" text not null
);

alter table only public."ClothingItem"
    add constraint "ClothingItem_clothingSessionId_fkey"
    foreign key ("clothingSessionId") references public."ClothingSession"(id);
