./configure --prefix=/usr                   \
            --build=$(support/config.guess) \
            --host=$LFS_TGT                 \
            --without-bash-malloc

if [ "$?" -eq 1 ];
then
    exit $?
fi

make && make DESTDIR=$LFS install

rm $LFS/bin/sh
ln -sv bash $LFS/bin/sh
